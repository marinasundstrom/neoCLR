// Matched connection-close load fixture, separate from the 32-request Go pilot.
package main

import (
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"strconv"
	"sync"
	"sync/atomic"
	"time"
)

type limitedListener struct {
	net.Listener
	slots       chan struct{}
	done        chan struct{}
	once        sync.Once
	connections sync.WaitGroup
	closed      atomic.Int64
}

type countedConn struct {
	net.Conn
	owner *limitedListener
	once  sync.Once
}

func (l *limitedListener) Accept() (net.Conn, error) {
	select {
	case l.slots <- struct{}{}:
	case <-l.done:
		return nil, net.ErrClosed
	}
	conn, err := l.Listener.Accept()
	if err != nil {
		<-l.slots
		return nil, err
	}
	l.connections.Add(1)
	return &countedConn{Conn: conn, owner: l}, nil
}

func (l *limitedListener) Close() error {
	l.once.Do(func() { close(l.done); l.Listener.Close() })
	return nil
}

func (c *countedConn) Close() error {
	err := c.Conn.Close()
	c.once.Do(func() {
		c.owner.closed.Add(1)
		<-c.owner.slots
		c.owner.connections.Done()
	})
	return err
}

func main() {
	if len(os.Args) != 2 {
		panic("expected connection limit")
	}
	limit, err := strconv.Atoi(os.Args[1])
	if err != nil || limit < 1 {
		panic("invalid connection limit")
	}
	tcp, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	listener := &limitedListener{Listener: tcp, slots: make(chan struct{}, limit), done: make(chan struct{})}
	control, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	var failed atomic.Int64
	server := &http.Server{
		ReadHeaderTimeout: 5 * time.Second,
		WriteTimeout:      5 * time.Second,
		MaxHeaderBytes:    8192,
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Match the Raven fixture's awaited queued operation.
			resumed := make(chan int, 1)
			go func() { resumed <- 1 }()
			if <-resumed != 1 {
				panic("handler continuation failed")
			}
			size := 10
			if r.URL.Path == "/stop" {
				size = 0
			} else if r.URL.Path == "/bulk" {
				size = 8192
			}
			body := make([]byte, size)
			for i := range body {
				body[i] = 'x'
			}
			// Suppress automatic headers absent from the neoCLR fixture.
			w.Header()["Date"] = nil
			w.Header()["Content-Type"] = nil
			w.Header().Set("Content-Length", strconv.Itoa(size))
			w.Header().Set("Connection", "close")
			if _, err := w.Write(body); err != nil {
				failed.Add(1)
			}
		}),
	}
	server.SetKeepAlivesEnabled(false)
	stopped := make(chan error, 1)
	go func() { stopped <- server.Serve(listener) }()
	fmt.Println(tcp.Addr().(*net.TCPAddr).Port)
	fmt.Println(control.Addr().(*net.TCPAddr).Port)
	signal, err := control.Accept()
	if err != nil {
		panic(err)
	}
	signal.Close()
	control.Close()
	// The controller signals only after response validation and draining finish.
	if err := server.Close(); err != nil {
		panic(err)
	}
	if err := <-stopped; !errors.Is(err, http.ErrServerClosed) {
		panic(err)
	}
	listener.connections.Wait()
	fmt.Println("DONE")
	fmt.Println(listener.closed.Load() - 1) // Exclude /stop; no control socket here.
	fmt.Println(0)
	fmt.Println(failed.Load())
}
