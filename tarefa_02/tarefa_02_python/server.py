import socket
import threading
import os
import hashlib
import sys

class ServidorTCP:
    def __init__(self, porta=8080):
        self.porta = porta
        self.servidor_socket = None
        self.clientes_conectados = []
        self.lock_clientes = threading.Lock()
        self.pasta_arquivos = "server_files"
        
        # Criar pasta de arquivos se não existir
        if not os.path.exists(self.pasta_arquivos):
            os.makedirs(self.pasta_arquivos)
            print(f"Pasta '{self.pasta_arquivos}' criada.")
    
    def calcular_sha256(self, caminho_arquivo):
        """Calcula o hash SHA-256 de um arquivo"""
        sha256_hash = hashlib.sha256()
        try:
            with open(caminho_arquivo, "rb") as f:
                for chunk in iter(lambda: f.read(8192), b""):
                    sha256_hash.update(chunk)
            return sha256_hash.hexdigest()
        except Exception as e:
            print(f"Erro ao calcular SHA-256: {e}")
            return None
    
    def enviar_arquivo(self, cliente_socket, nome_arquivo):
        """Envia um arquivo para o cliente"""
        caminho_arquivo = os.path.join(self.pasta_arquivos, nome_arquivo)
        
        try:
            if not os.path.exists(caminho_arquivo):
                # Arquivo não encontrado
                resposta = "ERRO ARQUIVO_NAO_ENCONTRADO"
                cliente_socket.send(resposta.encode('utf-8'))
                return
            
            # Calcular informações do arquivo
            tamanho_arquivo = os.path.getsize(caminho_arquivo)
            hash_arquivo = self.calcular_sha256(caminho_arquivo)
            
            if hash_arquivo is None:
                resposta = "ERRO FALHA_CALCULO_HASH"
                cliente_socket.send(resposta.encode('utf-8'))
                return
            
            # Enviar metadados
            metadados = f"OK {nome_arquivo} {tamanho_arquivo} {hash_arquivo}"
            cliente_socket.send(metadados.encode('utf-8'))
            
            # Aguardar confirmação do cliente
            confirmacao = cliente_socket.recv(1024).decode('utf-8').strip()
            if confirmacao != "PRONTO":
                print("CLIENTE NAO PRONTO")
                return
            
            print("ENVIANDO ARQUIVO")
            # Enviar conteúdo do arquivo
            with open(caminho_arquivo, 'rb') as arquivo:
                bytes_enviados = 0
                while bytes_enviados < tamanho_arquivo:
                    chunk = arquivo.read(8192)  # 8KB chunks
                    if not chunk:
                        break
                    cliente_socket.send(chunk)
                    bytes_enviados += len(chunk)
            
            print(f"Arquivo '{nome_arquivo}' enviado com sucesso ({bytes_enviados} bytes)")
            
        except Exception as e:
            print(f"Erro ao enviar arquivo: {e}")
            try:
                resposta = "ERRO FALHA_ENVIO"
                cliente_socket.send(resposta.encode('utf-8'))
            except:
                pass
    
    def broadcast_chat(self, mensagem, remetente_socket=None):
        """Envia mensagem de chat para todos os clientes conectados"""
        with self.lock_clientes:
            clientes_desconectados = []
            for cliente_socket in self.clientes_conectados:
                if cliente_socket != remetente_socket:
                    try:
                        msg_chat = f"CHAT_SERVER {mensagem}"
                        cliente_socket.send(msg_chat.encode('utf-8'))
                    except:
                        clientes_desconectados.append(cliente_socket)
            
            # Remover clientes desconectados
            for cliente in clientes_desconectados:
                self.clientes_conectados.remove(cliente)
    
    def processar_cliente(self, cliente_socket, endereco_cliente):
        """Thread para processar requisições de um cliente"""
        print(f"Cliente {endereco_cliente} conectado")
        
        # Adicionar cliente à lista
        with self.lock_clientes:
            self.clientes_conectados.append(cliente_socket)
        
        try:
            while True:
                # Receber requisição do cliente
                dados = cliente_socket.recv(1024).decode('utf-8').strip()
                if not dados:
                    break
                
                print(f"Requisição de {endereco_cliente}: {dados}")
                
                # Processar comando
                partes = dados.split(' ', 1)
                comando = partes[0].upper()
                
                if comando == "SAIR":
                    print(f"Cliente {endereco_cliente} solicitou desconexão")
                    break
                
                elif comando == "ARQUIVO":
                    if len(partes) < 2:
                        cliente_socket.send("ERRO NOME_ARQUIVO_OBRIGATORIO".encode('utf-8'))
                        continue
                    
                    nome_arquivo = partes[1].strip()
                    print(f"Cliente {endereco_cliente} solicitou arquivo: {nome_arquivo}")
                    self.enviar_arquivo(cliente_socket, nome_arquivo)
                
                elif comando == "CHAT":
                    if len(partes) < 2:
                        continue
                    
                    mensagem = partes[1].strip()
                    print(f"Chat de {endereco_cliente}: {mensagem}")
                    # self.broadcast_chat(f"[{endereco_cliente}]: {mensagem}", cliente_socket)
                
                else:
                    resposta = "ERRO COMANDO_INVALIDO"
                    cliente_socket.send(resposta.encode('utf-8'))
        
        except Exception as e:
            print(f"Erro na comunicação com {endereco_cliente}: {e}")
        
        finally:
            # Remover cliente da lista e fechar conexão
            with self.lock_clientes:
                if cliente_socket in self.clientes_conectados:
                    self.clientes_conectados.remove(cliente_socket)
            
            try:
                cliente_socket.close()
            except:
                pass
            
            print(f"Cliente {endereco_cliente} desconectado")
    
    def thread_console(self):
        """Thread para ler input do console e enviar para clientes"""
        while True:
            try:
                mensagem = input()
                if mensagem.strip():
                    self.broadcast_chat(f"SERVIDOR: {mensagem}")
            except KeyboardInterrupt:
                break
            except:
                break
    
    def iniciar(self):
        """Inicia o servidor"""
        try:
            # Criar socket TCP
            self.servidor_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.servidor_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            
            # Bind e listen
            self.servidor_socket.bind(('', self.porta))
            self.servidor_socket.listen(5)
            
            print(f"Servidor TCP iniciado na porta {self.porta}")
            print(f"Pasta de arquivos: {self.pasta_arquivos}")
            print("Digite mensagens para enviar para todos os clientes conectados")
            print("Aguardando conexões...\n")
            
            # Iniciar thread para console
            thread_console = threading.Thread(target=self.thread_console, daemon=True)
            thread_console.start()
            
            # Loop principal para aceitar conexões
            while True:
                try:
                    cliente_socket, endereco_cliente = self.servidor_socket.accept()
                    
                    # Criar thread para processar o cliente
                    thread_cliente = threading.Thread(
                        target=self.processar_cliente,
                        args=(cliente_socket, endereco_cliente),
                        daemon=True
                    )
                    thread_cliente.start()
                
                except KeyboardInterrupt:
                    print("\nEncerrando servidor...")
                    break
                except Exception as e:
                    print(f"Erro ao aceitar conexão: {e}")
        
        except Exception as e:
            print(f"Erro ao iniciar servidor: {e}")
        
        finally:
            self.parar()
    
    def parar(self):
        """Para o servidor e fecha todas as conexões"""
        print("Fechando conexões...")
        
        # Fechar todas as conexões de clientes
        with self.lock_clientes:
            for cliente_socket in self.clientes_conectados:
                try:
                    cliente_socket.close()
                except:
                    pass
            self.clientes_conectados.clear()
        
        # Fechar socket do servidor
        if self.servidor_socket:
            try:
                self.servidor_socket.close()
            except:
                pass
        
        print("Servidor encerrado.")

if __name__ == "__main__":
    servidor = ServidorTCP(8080)
    try:
        servidor.iniciar()
    except KeyboardInterrupt:
        print("\nEncerrando...")
    finally:
        servidor.parar()
