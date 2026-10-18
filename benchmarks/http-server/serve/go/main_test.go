package main

import (
	"errors"
	"net"
	"sync/atomic"
	"testing"
	"time"
)

type fakeListener struct{ accepted atomic.Int64 }

func (l *fakeListener) Accept() (net.Conn, error) {
	a, b := net.Pipe()
	b.Close()
	l.accepted.Add(1)
	return a, nil
}
func (l *fakeListener) Close() error   { return nil }
func (l *fakeListener) Addr() net.Addr { return nil }

func TestBudgetAndClose(t *testing.T) {
	base := &fakeListener{}
	l := &limitedListener{Listener: base, slots: make(chan struct{}, 2), done: make(chan struct{})}
	a, _ := l.Accept()
	b, _ := l.Accept()
	third := make(chan net.Conn, 1)
	go func() { c, _ := l.Accept(); third <- c }()
	select {
	case <-third:
		t.Fatal("accepted beyond connection budget")
	case <-time.After(20 * time.Millisecond):
	}
	if base.accepted.Load() != 2 {
		t.Fatal("underlying accept escaped budget")
	}
	a.Close()
	a.Close() // Must release and count exactly once.
	var c net.Conn
	select {
	case c = <-third:
	case <-time.After(time.Second):
		t.Fatal("released slot did not admit waiter")
	}
	stopped := make(chan error, 1)
	go func() { _, err := l.Accept(); stopped <- err }()
	l.Close()
	select {
	case err := <-stopped:
		if !errors.Is(err, net.ErrClosed) {
			t.Fatalf("unexpected close result: %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("close did not unblock admission")
	}
	b.Close()
	c.Close()
	l.connections.Wait()
	if l.closed.Load() != 3 || len(l.slots) != 0 {
		t.Fatal("unbalanced connection accounting")
	}
}
