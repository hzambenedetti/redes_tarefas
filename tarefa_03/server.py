import socket
import threading
import os
import sys
import signal
import datetime
from urllib.parse import unquote

class HTTPServer:
    def __init__(self, host='localhost', port=8080, max_connections=50):
        self.host = host
        self.port = port
        self.max_connections = max_connections
        self.files_directory = 'files'
        self.server_socket = None
        self.running = False
        self.active_connections = 0
        self.lock = threading.Lock()
        
        if not os.path.exists(self.files_directory):
            os.makedirs(self.files_directory)
            print(f"[INFO] Diretório '{self.files_directory}' criado")

    def get_content_type(self, file_path):
        if file_path.endswith('.html') or file_path.endswith('.htm'):
            return 'text/html; charset=utf-8'
        elif file_path.endswith('.jpg') or file_path.endswith('.jpeg'):
            return 'image/jpeg'
        elif file_path.endswith('.png'):
            return 'image/png'
        elif file_path.endswith('.gif'):
            return 'image/gif'
        elif file_path.endswith('.css'):
            return 'text/css'
        elif file_path.endswith('.js'):
            return 'application/javascript'
        else:
            return 'application/octet-stream'

    def parse_http_request(self, request_data):
        try:
            request_lines = request_data.decode('utf-8').split('\r\n')
            if not request_lines:
                return None, None, None, None
            
            request_line = request_lines[0]
            parts = request_line.split(' ')
            if len(parts) < 3:
                return None, None, None, None
            
            method = parts[0]
            path = parts[1]
            
            headers = {}
            body_start = -1
            for i, line in enumerate(request_lines[1:], 1):
                if line == '':
                    body_start = i + 1
                    break
                if ':' in line:
                    key, value = line.split(':', 1)
                    headers[key.strip().lower()] = value.strip()
            
            body = ''
            if body_start > 0 and body_start < len(request_lines):
                body = '\r\n'.join(request_lines[body_start:])
            
            return method, path, headers, body
            
        except Exception as e:
            print(f"[ERROR] Erro ao fazer parse da requisição: {e}")
            return None, None, None, None

    def handle_get_request(self, path):
        if '?' in path:
            path = path.split('?')[0]
        
        path = unquote(path)
        
        if path == '/':
            path = '/index.html'
        
        if path.startswith('/'):
            path = path[1:]
        
        file_path = os.path.join(self.files_directory, path)
        
        if not os.path.exists(file_path) or not os.path.isfile(file_path):
            return self.create_404_response()
        
        try:
            content_type = self.get_content_type(file_path)
            is_binary = content_type.startswith('image/') or content_type == 'application/octet-stream'
            
            mode = 'rb' if is_binary else 'r'
            encoding = None if is_binary else 'utf-8'
            
            with open(file_path, mode, encoding=encoding) as f:
                content = f.read()
            
            if is_binary:
                content_length = len(content)
                response = f"HTTP/1.1 200 OK\r\n"
                response += f"Content-Type: {content_type}\r\n"
                response += f"Content-Length: {content_length}\r\n"
                response += f"Date: {datetime.datetime.now().strftime('%a, %d %b %Y %H:%M:%S GMT')}\r\n"
                response += "Connection: close\r\n"
                response += "\r\n"
                
                return response.encode('utf-8') + content
            else:
                content_bytes = content.encode('utf-8')
                content_length = len(content_bytes)
                
                response = f"HTTP/1.1 200 OK\r\n"
                response += f"Content-Type: {content_type}\r\n"
                response += f"Content-Length: {content_length}\r\n"
                response += f"Date: {datetime.datetime.now().strftime('%a, %d %b %Y %H:%M:%S GMT')}\r\n"
                response += "Connection: close\r\n"
                response += "\r\n"
                
                return response.encode('utf-8') + content_bytes
                
        except Exception as e:
            print(f"[ERROR] Erro ao ler arquivo {file_path}: {e}")
            return self.create_404_response()

    def handle_post_request(self, path, headers, body):
        print(f"[POST] Dados recebidos em {path}:")
        print(f"[POST] Headers: {headers}")
        print(f"[POST] Body: {body}")
        
        # Resposta simples para POST
        response_content = """<!DOCTYPE html>
<html>
<head>
    <title>POST Recebido</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>POST Recebido com Sucesso!</h1>
    <p>Os dados foram logados no servidor.</p>
    <a href="/index.html">Voltar ao início</a>
</body>
</html>"""
        
        content_bytes = response_content.encode('utf-8')
        content_length = len(content_bytes)
        
        response = f"HTTP/1.1 200 OK\r\n"
        response += f"Content-Type: text/html; charset=utf-8\r\n"
        response += f"Content-Length: {content_length}\r\n"
        response += f"Date: {datetime.datetime.now().strftime('%a, %d %b %Y %H:%M:%S GMT')}\r\n"
        response += "Connection: close\r\n"
        response += "\r\n"
        
        return response.encode('utf-8') + content_bytes

    def create_404_response(self):
        content = """<!DOCTYPE html>
<html>
<head>
    <title>404 - Não Encontrado</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>404 - Arquivo Não Encontrado</h1>
    <p>O arquivo solicitado não foi encontrado no servidor.</p>
    <a href="/index.html">Voltar ao início</a>
</body>
</html>"""
        
        content_bytes = content.encode('utf-8')
        content_length = len(content_bytes)
        
        response = f"HTTP/1.1 404 Not Found\r\n"
        response += f"Content-Type: text/html; charset=utf-8\r\n"
        response += f"Content-Length: {content_length}\r\n"
        response += f"Date: {datetime.datetime.now().strftime('%a, %d %b %Y %H:%M:%S GMT')}\r\n"
        response += "Connection: close\r\n"
        response += "\r\n"
        
        return response.encode('utf-8') + content_bytes

    def handle_client(self, client_socket, client_address):
        try:
            with self.lock:
                self.active_connections += 1
            
            print(f"[CONN] Nova conexão de {client_address[0]}:{client_address[1]} (Total: {self.active_connections})")
            
            client_socket.settimeout(30.0)
            request_data = b''
            
            while True:
                try:
                    chunk = client_socket.recv(4096)
                    if not chunk:
                        break
                    request_data += chunk
                    
                    if b'\r\n\r\n' in request_data:
                        break
                        
                except socket.timeout:
                    print(f"[WARN] Timeout na conexão {client_address}")
                    break
            
            if not request_data:
                return
            
            method, path, headers, body = self.parse_http_request(request_data)
            
            if method is None:
                print(f"[ERROR] Requisição inválida de {client_address}")
                return
            
            print(f"[REQ] {method} {path} de {client_address[0]}:{client_address[1]}")
            
            if method == 'GET':
                response = self.handle_get_request(path)
            elif method == 'POST':
                response = self.handle_post_request(path, headers, body)
            else:
                # Método não suportado
                response = self.create_404_response()
            
            client_socket.sendall(response)
            
        except Exception as e:
            print(f"[ERROR] Erro ao tratar cliente {client_address}: {e}")
        
        finally:
            try:
                client_socket.close()
            except:
                pass
            
            with self.lock:
                self.active_connections -= 1
            
            print(f"[DISC] Conexão {client_address[0]}:{client_address[1]} encerrada (Total: {self.active_connections})")

    def start(self):
        try:
            self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            
            self.server_socket.bind((self.host, self.port))
            self.server_socket.listen(self.max_connections)
            
            self.running = True
            
            print(f"[INFO] Servidor HTTP iniciado em {self.host}:{self.port}")
            
            while self.running:
                try:
                    client_socket, client_address = self.server_socket.accept()
                    
                    if self.active_connections >= self.max_connections:
                        print(f"[WARN] Limite de conexões atingido. Rejeitando {client_address}")
                        client_socket.close()
                        continue
                    
                    client_thread = threading.Thread(
                        target=self.handle_client,
                        args=(client_socket, client_address),
                        daemon=True
                    )
                    client_thread.start()
                    
                except OSError:
                    if self.running:
                        print("[ERROR] Erro ao aceitar conexão")
                    break
                    
        except Exception as e:
            print(f"[ERROR] Erro ao iniciar servidor: {e}")
        
        finally:
            self.stop()

    def stop(self):
        print("\n[INFO] Parando servidor...")
        self.running = False
        
        if self.server_socket:
            try:
                self.server_socket.close()
            except:
                pass
        
        wait_time = 0
        while self.active_connections > 0 and wait_time < 5:
            print(f"[INFO] Aguardando {self.active_connections} conexões ativas...")
            threading.Event().wait(1)
            wait_time += 1
        
        print("[INFO] Servidor parado.")

def signal_handler(sig, frame):
    print("\n[INFO] Recebido sinal de interrupção...")
    if 'server' in globals():
        server.stop()
    sys.exit(0)

def main():
    global server
    
    signal.signal(signal.SIGINT, signal_handler)
    
    server = HTTPServer()
    
    try:
        server.start()
    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    main()
