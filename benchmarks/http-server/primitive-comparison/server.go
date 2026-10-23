package main

import (
	"fmt"
	"net"
	"net/http"
	"strconv"
)

func main() {
	listener, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	control, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	body := []byte("xxxxxxxxxx")
	server := &http.Server{Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Length", strconv.Itoa(len(body)))
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		if _, err := w.Write(body); err != nil {
			panic(err)
		}
	})}
	done := make(chan error, 1)
	go func() { done <- server.Serve(listener) }()
	fmt.Printf("READY %d %d\n", listener.Addr().(*net.TCPAddr).Port, control.Addr().(*net.TCPAddr).Port)
	signal, err := control.Accept()
	if err != nil {
		panic(err)
	}
	signal.Close()
	control.Close()
	server.Close()
	if err := <-done; err != http.ErrServerClosed {
		panic(err)
	}
}
