package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"strconv"
	"sync/atomic"
	"time"
)

type person struct {
	ID     int    `json:"id"`
	Name   string `json:"name"`
	Active bool   `json:"active"`
}

func main() {
	if len(os.Args) != 2 || (os.Args[1] != "plain" && os.Args[1] != "json") {
		panic("choose plain or json")
	}
	workload := os.Args[1]
	listener, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	var closed atomic.Int32
	completed := make(chan struct{})
	server := &http.Server{
		// Direct HandlerFunc, no ServeMux or routing framework.
		Handler: http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			var body []byte
			contentType := "text/plain; charset=utf-8"
			if workload == "plain" {
				body = []byte("Café 🌍")
			} else {
				var err error
				body, err = json.Marshal(person{ID: 42, Name: "Café team", Active: true})
				if err != nil {
					panic(err)
				}
				contentType = "application/json; charset=utf-8"
			}
			w.Header().Set("Content-Type", contentType)
			w.Header().Set("Content-Length", strconv.Itoa(len(body)))
			w.Header().Set("Connection", "close")
			if _, err := w.Write(body); err != nil {
				panic(err)
			}
		}),
		// Count closed connections, after response output has completed.
		ConnState: func(_ net.Conn, state http.ConnState) {
			if state == http.StateClosed && closed.Add(1) == 32 {
				close(completed)
			}
		},
	}
	stopped := make(chan error, 1)
	go func() { stopped <- server.Serve(listener) }()
	fmt.Println(listener.Addr().(*net.TCPAddr).Port)
	select {
	case <-completed:
	case <-time.After(120 * time.Second):
		panic("benchmark requests did not complete")
	}
	if err := server.Close(); err != nil {
		panic(err)
	}
	if err := <-stopped; !errors.Is(err, http.ErrServerClosed) {
		panic(err)
	}
	fmt.Println("DONE")
}
