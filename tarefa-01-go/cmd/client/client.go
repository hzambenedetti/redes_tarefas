package main

import (
    "crypto/sha256"
    "encoding/binary"
    "flag"
    "fmt"
    "log"
    "math/rand"
    "net"
    "os"
    "path/filepath"
    "time"
)

const (
    TypeGET  = 1
    TypeDATA = 2
    TypeACK  = 3
    TypeEOR  = 4
    TypeNOTFOUND = 5
    HeaderSize      = 1 + 1 + 2 + 32
    DefaultDownload = "download"
)

var (
    addr       = flag.String("addr", "localhost:9000", "server address")
    timeoutMs  = flag.Int("timeout", 500, "ACK timeout in ms")
    dropRate   = flag.Int("droprate", 10, "packet drop % for simulation")
    maxRetries = flag.Int("maxretries", 10, "max retries per packet")
    maxPayload = flag.Int("payload", 1024, "max payload size per packet")
    fileName   = flag.String("file", "teste.jpg", "file to download")
)

func main() {
		clientInstance()
}

func clientInstance(){
    flag.Parse()
    if *fileName == "" {
        fmt.Println("Usage: client -file <filename> [flags]")
        os.Exit(1)
    }
    log.SetFlags(0)
    rand.Seed(time.Now().UnixNano())

    addrUDP, err := net.ResolveUDPAddr("udp", *addr)
    if err != nil {
        log.Fatalf("[%s] ResolveUDPAddr error: %v", timestamp(), err)
    }
    conn, err := net.DialUDP("udp", nil, addrUDP)
    if err != nil {
        log.Fatalf("[%s] DialUDP error: %v", timestamp(), err)
    }
    defer conn.Close()

    // Send GET
    sendGET(conn, *fileName)

    // Prepare output
    outPath := filepath.Join(DefaultDownload, *fileName)

    expectedBit := byte(0)
    fullData := []byte{}
    timeout := time.Duration(*timeoutMs) * time.Millisecond
    retries := 0
    var expectedFullHash [32]byte
    isEOR := false
    for !isEOR{
        buf := make([]byte, HeaderSize+*maxPayload)
        conn.SetReadDeadline(time.Now().Add(timeout))
        _, err := conn.Read(buf)
        if err != nil {
            if retries++; retries > *maxRetries {
                log.Fatalf("[%s] Max retries reached, aborting", timestamp())
            }
            continue
        }
        retries = 0
        t := buf[0]
        bit := buf[1]
        length := int(binary.BigEndian.Uint16(buf[2:4]))
        hash := buf[4:36]
        payload := buf[36:36+length]

        switch t {
        case TypeDATA:
            calc := sha256.Sum256(payload)
            if bit != expectedBit || !equal(hash, calc[:]) {
                sendACK(conn, expectedBit^1) // NACK by sending old bit
                continue
            }
            fullData = append(fullData, payload...)
            sendACK(conn, bit)
            log.Printf("[%s] SENT ACK bit=%d", timestamp(), bit)
            expectedBit ^= 1

        case TypeEOR:
            // verify full-file hash
            isEOR = true
            copy(expectedFullHash[:], buf[4:36])
            sendACK(conn, bit)
            log.Printf("[%s] SENT final ACK bit=%d, download complete: %s", timestamp(), bit, outPath)
            break

        case TypeNOTFOUND:
          log.Fatalf("[%s] File not found", timestamp())
          break 
        }
		}

  fullHash := sha256.Sum256(fullData)
  if !equal(fullHash[:], expectedFullHash[:]) {
    log.Fatalf("[%s] Full-file hash mismatch ", timestamp())
    return
  }
  outFile, err := os.Create(outPath)
  if err != nil {
    log.Fatalf("[%s] Create file error: %v", timestamp(), err)
  }
  defer outFile.Close()
  log.Fatalf("[%s] Saving file %s", timestamp(), outPath)
  outFile.Write(fullData)
    
}

func sendGET(conn *net.UDPConn, filename string) {
    buf := make([]byte, HeaderSize+len(filename))
    buf[0] = TypeGET
    buf[1] = 0
    binary.BigEndian.PutUint16(buf[2:4], uint16(len(filename)))
    copy(buf[4:], []byte(filename))
    conn.Write(buf)
    log.Printf("[%s] Sent GET %s", timestamp(), filename)
}

func sendACK(conn *net.UDPConn, bit byte) {
    buf := make([]byte, HeaderSize)
    buf[0] = TypeACK
    buf[1] = bit
    conn.Write(buf)
}

func equal(a, b []byte) bool {
    if len(a) != len(b) { return false }
    for i := range a {
        if a[i] != b[i] { return false }
    }
    return true
}

func timestamp() string {
    return time.Now().Format(time.RFC3339)
}
