package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"math"
	"net"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"
)

func cpu() float64 {
	var r syscall.Rusage
	if err := syscall.Getrusage(syscall.RUSAGE_SELF, &r); err != nil {
		panic(err)
	}
	return float64(r.Utime.Sec+r.Stime.Sec) + float64(r.Utime.Usec+r.Stime.Usec)/1e6
}
func main() {
	if len(os.Args) != 3 {
		panic("expected port and seconds")
	}
	seconds, err := strconv.ParseFloat(os.Args[2], 64)
	if err != nil || seconds <= 0 {
		panic("invalid duration")
	}
	type result struct {
		latency []float64
		drained int
		err     error
	}
	results := make(chan result, 32)
	startCPU := cpu()
	start := time.Now()
	stop := start.Add(time.Duration(seconds * float64(time.Second)))
	var group sync.WaitGroup
	for lane := 0; lane < 32; lane++ {
		group.Add(1)
		go func() {
			defer group.Done()
			r := result{}
			for time.Now().Before(stop) {
				began := time.Now()

				connection, err := net.DialTimeout("tcp", "127.0.0.1:"+os.Args[1], 10*time.Second)
				if err != nil {
					r.err = err
					break
				}
				err = connection.SetDeadline(time.Now().Add(10 * time.Second))
				if err == nil {
					_, err = connection.Write([]byte("GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"))
				}
				reader := bufio.NewReaderSize(connection, 512)
				status := ""
				if err == nil {
					status, err = reader.ReadString('\n')
				}
				length, closeHeader, headerBytes := -1, false, len(status)
				for err == nil {
					line, readErr := reader.ReadString('\n')
					err = readErr
					headerBytes += len(line)
					if headerBytes > 8192 {
						err = fmt.Errorf("oversized headers")
						break
					}
					if err != nil || line == "\r\n" {
						break
					}
					key, value, ok := strings.Cut(line, ":")
					if !ok {
						err = fmt.Errorf("invalid header")
						break
					}
					switch strings.ToLower(key) {
					case "content-length":
						length, err = strconv.Atoi(strings.TrimSpace(value))
					case "connection":
						closeHeader = strings.EqualFold(strings.TrimSpace(value), "close")
					}
				}
				var body []byte
				if err == nil {
					// Read through server EOF before closing locally. Otherwise
					// early client closes can exhaust ephemeral ports in TIME_WAIT.
					body, err = io.ReadAll(io.LimitReader(reader, 11))
				}
				connection.Close()
				if err != nil || !strings.HasPrefix(status, "HTTP/1.1 200 ") || length != 10 ||
					!bytes.Equal(body, []byte("xxxxxxxxxx")) || !closeHeader {
					r.err = fmt.Errorf("invalid response: status=%q length=%d body=%q close=%v error=%v",
						status, length, body, closeHeader, err)
					break
				}
				finished := time.Now()
				if !finished.After(stop) {
					r.latency = append(r.latency, float64(finished.Sub(began))/1e6)
				} else {
					r.drained++
				}
			}
			results <- r
		}()
	}
	group.Wait()
	elapsed := time.Since(start).Seconds()
	usedCPU := cpu() - startCPU
	close(results)
	var latencies []float64
	drained := 0
	for r := range results {
		if r.err != nil {
			panic(r.err)
		}
		latencies = append(latencies, r.latency...)
		drained += r.drained
	}
	if len(latencies) == 0 {
		panic("no completed responses")
	}
	sort.Float64s(latencies)
	percentiles := map[string]float64{}
	for name, q := range map[string]float64{"p50": .5, "p95": .95, "p99": .99} {
		percentiles[name] = latencies[int(math.Ceil(float64(len(latencies))*q))-1]
	}
	output := map[string]any{"completed": len(latencies), "drained": drained, "requestsPerSecond": float64(len(latencies)) / seconds,
		"latencyMilliseconds": percentiles, "clientCpuSeconds": usedCPU, "windowAndDrainSeconds": elapsed,
		"clientCpuPerWall": usedCPU / elapsed, "clientProcessCpuSeconds": cpu()}
	if err := json.NewEncoder(os.Stdout).Encode(output); err != nil {
		panic(err)
	}
}
