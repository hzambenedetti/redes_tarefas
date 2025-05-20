package main

import (
    "crypto/sha256"
    "encoding/binary"
    "flag"
    "io"
    "log"
    "net"
    "os"
    "time"
)

// Packet types
const (
    TypeGET  = 1
    TypeDATA = 2
    TypeACK  = 3
    TypeEOR  = 4
    TypeNOTFOUND = 5
)

// Header lengths
const (
    HeaderSize       = 1 + 1 + 2 + 32 // type + seqBit + length + hash
		resourcePath 		 = "resources/"
)

// Configurable via flags
var (
    addr       = flag.String("addr", ":9000", "server listen address")
    timeoutMs  = flag.Int("timeout", 500, "ACK timeout in ms")
    maxRetries = flag.Int("maxretries", 10, "max retries per packet")
    maxPayload = flag.Int("payload", 1024, "max payload size per packet")
)

func main() {
    flag.Parse()
    log.SetFlags(0)

    addrUDP, err := net.ResolveUDPAddr("udp", *addr)
    if err != nil {
        log.Fatalf("[%s] ResolveUDPAddr error: %v", timestamp(), err)
    }
    conn, err := net.ListenUDP("udp", addrUDP)
    if err != nil {
        log.Fatalf("[%s] ListenUDP error: %v", timestamp(), err)
    }
    defer conn.Close()
    log.Printf("[%s] Server listening on %s", timestamp(), *addr)

		buf := make([]byte, HeaderSize+*maxPayload)
    for {
        n, clientAddr, err := conn.ReadFromUDP(buf)
        if err != nil {
            log.Printf("[%s] Read error: %v", timestamp(), err)
            continue
        }
        if n < HeaderSize || buf[0] != TypeGET {
            continue
        }
        // Handle single client, block until done
        serveClient(conn, clientAddr, buf[:n])
				conn.SetReadDeadline(time.Time{}) // disable timeout
    }
}

func serveClient(conn *net.UDPConn, client *net.UDPAddr, req []byte) {
    // Parse GET
    length := int(binary.BigEndian.Uint16(req[2:4]))
    filename := string(req[4:4+length])
    path := resourcePath + filename 

    f, err := os.Open(path)
    if err != nil {
        log.Printf("[%s] File not found: %s", timestamp(), path)
        sendNOTFOUND(conn, client)
        return
    }
    defer f.Close()

    data, err := io.ReadAll(f)
    if err != nil {
        log.Printf("[%s] Read error: %v", timestamp(), err)
        return
    }
    fullHash := sha256.Sum256(data)

    seqBit := byte(0)
    timeout := time.Duration(*timeoutMs) * time.Millisecond

    // Send each segment stop-and-wait
    for offset := 0; offset < len(data); offset += *maxPayload {
        end := min(offset + *maxPayload, len(data))
        payload := data[offset:end]
        hash := sha256.Sum256(payload)

        pkt := make([]byte, HeaderSize+len(payload))
        pkt[0] = TypeDATA
        pkt[1] = seqBit
        binary.BigEndian.PutUint16(pkt[2:4], uint16(len(payload)))
        copy(pkt[4:36], hash[:])
        copy(pkt[36:], payload)

        retries := 0
        for {
						conn.WriteToUDP(pkt, client)
						log.Printf("[%s] SENT DATA bit=%d size=%d", timestamp(), seqBit, len(payload))

            conn.SetReadDeadline(time.Now().Add(timeout))
            ackBuf := make([]byte, HeaderSize)
            n, _, err := conn.ReadFromUDP(ackBuf)
            if err == nil && n >= HeaderSize && ackBuf[0] == TypeACK && ackBuf[1] == seqBit {
                log.Printf("[%s] RECV ACK bit=%d", timestamp(), seqBit)
                seqBit ^= 1
                break
            }
            retries++
            if retries > *maxRetries {
                log.Printf("[%s] Max retries reached, aborting %s", timestamp(), filename)
                return
            }
        }
    }

    // Send EOR with full-file hash
		sendEOR(conn, client, fullHash, seqBit)
		
    // Wait final ACK
    conn.SetReadDeadline(time.Now().Add(timeout))
    ackBuf := make([]byte, HeaderSize)
    n, _, _ := conn.ReadFromUDP(ackBuf)
    if n >= HeaderSize && ackBuf[0] == TypeACK && ackBuf[1] == seqBit {
        log.Printf("[%s] RECV final ACK bit=%d, transfer complete", timestamp(), seqBit)
    }
}

func sendEOR(conn *net.UDPConn, client *net.UDPAddr, fullHash [32]byte, seqBit byte){
    pkt := make([]byte, HeaderSize)
    pkt[0] = TypeEOR
    pkt[1] = seqBit
    copy(pkt[4:36], fullHash[:])
    conn.WriteToUDP(pkt, client)
    log.Printf("[%s] SENT EOR bit=%d", timestamp(), seqBit)
}

func sendNOTFOUND(conn *net.UDPConn, client *net.UDPAddr){
    pkt := make([]byte, HeaderSize)
    pkt[0] = TypeNOTFOUND
    conn.WriteToUDP(pkt, client)
}

func timestamp() string {
    return time.Now().Format(time.RFC3339)
}
