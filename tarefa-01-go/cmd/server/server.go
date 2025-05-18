package main

import (
	"encoding/binary"
	"fmt"
	"hash/crc32"
	"io"
	"net"
	"os"
	"strings"
	"time"
)

const (
	HeaderSize  = 9 // seqBit (1) + checksum (4) + data length (4)
	MaxDataSize = 1400
	ServerPort  = 9000
	Timeout     = 2 * time.Second
	MaxRetries  = 5
	resourcePath = "/home/henrique/Documents/Faculdade/2025_1/redes_de_computadores/redes_tarefas/tarefa-01-go/resources/"
)

func main() {
	addr := net.UDPAddr{Port: ServerPort}
	conn, err := net.ListenUDP("udp", &addr)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	fmt.Printf("[SERVER] Listening on port %d\n", ServerPort)

	buffer := make([]byte, MaxDataSize+HeaderSize)
	for {
		n, clientAddr, err := conn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Println("[SERVER] Read error:", err)
			continue
		}

		go handleClient(conn, clientAddr, buffer[:n])
	}
}

func handleClient(conn *net.UDPConn, clientAddr *net.UDPAddr, request []byte) {
	reqStr := string(request)
	if !strings.HasPrefix(reqStr, "GET ") {
		fmt.Printf("[SERVER] Invalid request from %s\n", clientAddr)
		return
	}

	filename := strings.TrimSpace(reqStr[4:])
	fmt.Printf("[SERVER] File request for '%s' from %s\n", filename, clientAddr)

	file, err := os.Open(resourcePath + filename)
	if err != nil {
		fmt.Printf("[SERVER] File not found: %s\n", filename)
		sendError(conn, clientAddr, "File not found")
		return
	}
	defer file.Close()

	// fileInfo, _ := file.Stat()
	// if fileInfo.Size() < 1<<20 {
	// 	sendError(conn, clientAddr, "File too small")
	// 	return
	// }

	var seqBit byte = 0
	retries := 0
	buf := make([]byte, MaxDataSize)

	for {
		n, err := file.Read(buf)
		if err == io.EOF {
			fmt.Printf("[SERVER] File transfer complete to %s\n", clientAddr)
			return
		}

		segment := createSegment(seqBit, buf[:n])
		fmt.Printf("[SERVER] Sending segment (seq=%d) to %s\n", seqBit, clientAddr)

		for retries < MaxRetries {
			sendSegment(conn, clientAddr, segment)
			ackReceived := waitForACK(conn, seqBit)

			if ackReceived {
				fmt.Printf("[SERVER] Received ACK %d from %s\n", seqBit, clientAddr)
				seqBit ^= 1 // Toggle sequence bit
				retries = 0
				break
			}

			retries++
			fmt.Printf("[SERVER] Timeout, resending segment (seq=%d) to %s (retry %d)\n", 
				seqBit, clientAddr, retries)
		}

		if retries >= MaxRetries {
			fmt.Printf("[SERVER] Max retries exceeded for %s\n", clientAddr)
			return
		}
	}
}

func createSegment(seqBit byte, data []byte) []byte {
	checksum := crc32.ChecksumIEEE(data)
	buf := make([]byte, HeaderSize+len(data))
	
	buf[0] = seqBit
	binary.BigEndian.PutUint32(buf[1:5], checksum)
	binary.BigEndian.PutUint32(buf[5:9], uint32(len(data)))
	copy(buf[9:], data)
	
	return buf
}

func sendSegment(conn *net.UDPConn, addr *net.UDPAddr, segment []byte) {
	_, err := conn.WriteToUDP(segment, addr)
	if err != nil {
		fmt.Println("[SERVER] Send error:", err)
	}
}

func waitForACK(conn *net.UDPConn, expectedSeq byte) bool {
	ackBuffer := make([]byte, 1)
	conn.SetReadDeadline(time.Now().Add(Timeout))
	
	for {
		n, _, err := conn.ReadFromUDP(ackBuffer)
		if err != nil {
			return false
		}

		if n == 1 && ackBuffer[0] == expectedSeq {
			return true
		}
	}
}

func sendError(conn *net.UDPConn, addr *net.UDPAddr, message string) {
	fmt.Printf("[SERVER] Sending error to %s: %s\n", addr, message)
	conn.WriteToUDP([]byte("ERROR: "+message), addr)
}
