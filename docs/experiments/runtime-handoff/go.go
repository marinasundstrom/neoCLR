package main

import (
    "encoding/json"
    "fmt"
    "io"
    "net"
    "os"
    "runtime"
    "strconv"
    "syscall"
    "time"
)

func check(err error) { if err != nil { panic(err) } }
func compute(iterations int) uint64 {
    x := uint64(1)
    for n := 0; n < iterations; n++ { x ^= x << 13; x ^= x >> 7; x ^= x << 17 }
    return x
}
func cpu() float64 {
    var r syscall.Rusage; check(syscall.Getrusage(syscall.RUSAGE_SELF, &r))
    return float64(r.Utime.Sec+r.Stime.Sec) + float64(r.Utime.Usec+r.Stime.Usec)/1e6
}
func main() {
    mode := os.Args[1]; rounds, err := strconv.Atoi(os.Args[2]); check(err)
    iterations, err := strconv.Atoi(os.Args[3]); check(err)
    if (mode != "idle" && mode != "inline" && mode != "offload") || rounds < 1 || iterations < 1 { panic("arguments") }
    runtime.GOMAXPROCS(2)
    runtime.LockOSThread() // Application owner affinity, not a claim of UI integration.
    defer runtime.UnlockOSThread()
    listener, err := net.ListenTCP("tcp4", &net.TCPAddr{IP: net.IPv4(127,0,0,1)}); check(err)
    fmt.Println(listener.Addr().(*net.TCPAddr).Port)
    socket, err := listener.AcceptTCP(); check(err); check(listener.Close())
    defer socket.Close(); check(socket.SetNoDelay(true))
    start := time.Now(); cpuStart := cpu(); checksum := uint64(0)
    for round := 0; round < rounds; round++ {
        received := make(chan error, 1)
        go func() {
            buffer := []byte{0}; _, err := io.ReadFull(socket, buffer)
            if err == nil && buffer[0] != 'x' { err = fmt.Errorf("payload") }
            received <- err
        }()
        n, err := socket.Write([]byte{'R'}); check(err); if n != 1 { panic("ready") }
        var work chan uint64
        if mode == "offload" { work = make(chan uint64, 1); go func() { work <- compute(iterations) }() }
        if mode == "inline" { checksum = compute(iterations) }
        check(<-received)
        n, err = socket.Write([]byte{'A'}); check(err); if n != 1 { panic("ack") }
        if work != nil { checksum = <-work }
    }
    check(json.NewEncoder(os.Stdout).Encode(map[string]interface{}{
        "mode":mode, "rounds":rounds, "iterations":iterations,
        "checksum":strconv.FormatUint(checksum,10), "seconds":time.Since(start).Seconds(),
        "cpuSeconds":cpu()-cpuStart, "runtime":runtime.Version(),
    }))
}
