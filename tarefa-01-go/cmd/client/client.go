package main

import (
	"encoding/binary"
	"fmt"
	"hash/crc32"
	"net"
	"os"
	"time"
	"strings"
)

const (
	HeaderSize  = 9
	MaxDataSize = 1400
	Timeout     = 3 * time.Second
	downloadPath = "/home/henrique/Documents/Faculdade/2025_1/redes_de_computadores/redes_tarefas/tarefa-01-go/downloads/"
)

func main() {
	// serverAddr := input("Enter server address (IP:Port): ")
	serverAddr := "localhost:9000" 
	// filename := input("Enter filename to download: ")
	filename := "teste.jpg" 

	conn, err := net.Dial("udp", serverAddr)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	fmt.Printf("[CLIENT] Sending request for '%s' to %s\n", filename, serverAddr)
	conn.Write([]byte("GET " + filename))

	outputFile := downloadPath + "received_" + filename
	file, err := os.Create(outputFile)
	if err != nil {
		fmt.Println("[CLIENT] File creation error:", err)
		return
	}
	defer file.Close()

	var expectedSeq byte = 0
	buffer := make([]byte, MaxDataSize+HeaderSize)

	for {
		conn.SetReadDeadline(time.Now().Add(Timeout))
		n, err := conn.Read(buffer)
		if err != nil {
			fmt.Println("[CLIENT] Transfer complete or timeout")
			break
		}

		if strings.HasPrefix(string(buffer[:n]), "ERROR: ") {
			fmt.Printf("[CLIENT] Error: %s\n", buffer[7:n])
			return
		}

		seqBit := buffer[0]
		checksum := binary.BigEndian.Uint32(buffer[1:5])
		dataLength := binary.BigEndian.Uint32(buffer[5:9])
		data := buffer[9 : 9+dataLength]

		fmt.Printf("[CLIENT] Received segment (seq=%d)\n", seqBit)

		if seqBit != expectedSeq {
			fmt.Printf("[CLIENT] Unexpected sequence %d, resending ACK %d\n", 
				seqBit, expectedSeq^1)
			conn.Write([]byte{expectedSeq ^ 1})
			continue
		}

		if crc32.ChecksumIEEE(data) != checksum {
			fmt.Printf("[CLIENT] Bad checksum in seq %d\n", seqBit)
			conn.Write([]byte{expectedSeq ^ 1})
			continue
		}

		fmt.Printf("[CLIENT] Sending ACK %d\n", seqBit)
		conn.Write([]byte{seqBit})
		file.Write(data)
		expectedSeq ^= 1
	}

	fmt.Printf("[CLIENT] File saved as %s\n", outputFile)
}

func input(prompt string) string {
	fmt.Print(prompt)
	var input string
	fmt.Scanln(&input)
	return input
}
