import socket
import threading
import os
import hashlib
import sys

class ClienteTCP:
    def __init__(self):
        self.cliente_socket = None
        self.conectado = False
        self.pasta_downloads = "downloads"
        # Criar pasta de downloads se não existir
        if not os.path.exists(self.pasta_downloads):
            os.makedirs(self.pasta_downloads)
            print(f"Pasta '{self.pasta_downloads}' criada.")
    
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
    
    def conectar(self, servidor_ip, servidor_porta):
        """Conecta ao servidor"""
        try:
            self.cliente_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.cliente_socket.connect((servidor_ip, servidor_porta))
            self.conectado = True
            print(f"Conectado ao servidor {servidor_ip}:{servidor_porta}")
            return True
        except Exception as e:
            print(f"Erro ao conectar: {e}")
            return False
    
    def receber_arquivo(self, nome_arquivo):
        """Recebe um arquivo do servidor"""
        try:
            # Enviar requisição
            requisicao = f"ARQUIVO {nome_arquivo}"
            self.cliente_socket.send(requisicao.encode('utf-8'))
            
            # Receber resposta inicial
            resposta = self.cliente_socket.recv(1024).decode('utf-8').strip()
            
            if resposta.startswith("ERRO"):
                if "ARQUIVO_NAO_ENCONTRADO" in resposta:
                    print(f"Erro: Arquivo '{nome_arquivo}' não encontrado no servidor")
                elif "FALHA_CALCULO_HASH" in resposta:
                    print("Erro: Falha ao calcular hash do arquivo no servidor")
                elif "FALHA_ENVIO" in resposta:
                    print("Erro: Falha durante o envio do arquivo")
                else:
                    print(f"Erro: {resposta}")
                return
            
            # Processar metadados
            partes = resposta.split(' ')
            if len(partes) < 4 or partes[0] != "OK":
                print("Erro: Resposta inválida do servidor")
                return
            
            status = partes[0]
            nome_arquivo_servidor = partes[1]
            tamanho_arquivo = int(partes[2])
            hash_servidor = partes[3]
            
            print(f"Recebendo arquivo: {nome_arquivo_servidor}")
            print(f"Tamanho: {tamanho_arquivo} bytes")
            print(f"Hash SHA-256: {hash_servidor}")
            
            # Confirmar que está pronto para receber
            self.cliente_socket.send("PRONTO".encode('utf-8'))
            
            # Receber conteúdo do arquivo
            caminho_download = os.path.join(self.pasta_downloads, nome_arquivo_servidor)
            
            with open(caminho_download, 'wb') as arquivo:
                bytes_recebidos = 0
                while bytes_recebidos < tamanho_arquivo:
                    chunk_size = min(8192, tamanho_arquivo - bytes_recebidos)
                    chunk = self.cliente_socket.recv(chunk_size)
                    if not chunk:
                        print("Erro: Conexão perdida durante a transferência")
                        return
                    
                    arquivo.write(chunk)
                    bytes_recebidos += len(chunk)
                    
                    # Mostrar progresso
                    progresso = (bytes_recebidos / tamanho_arquivo) * 100
                    print(f"\rProgresso: {progresso:.1f}% ({bytes_recebidos}/{tamanho_arquivo} bytes)", end='')
            
            print(f"\nArquivo salvo em: {caminho_download}")
            
            # Verificar integridade
            print("Verificando integridade do arquivo...")
            hash_recebido = self.calcular_sha256(caminho_download)
            
            if hash_recebido == hash_servidor:
                print("✓ Arquivo recebido com sucesso! Integridade verificada.")
            else:
                print("⚠ ATENÇÃO: Hash não confere! Arquivo pode estar corrompido.")
                print(f"Hash esperado: {hash_servidor}")
                print(f"Hash calculado: {hash_recebido}")
        
        except Exception as e:
            print(f"Erro ao receber arquivo: {e}")
    
    def enviar_chat(self, mensagem):
        """Envia mensagem de chat"""
        try:
            requisicao = f"CHAT {mensagem}"
            self.cliente_socket.send(requisicao.encode('utf-8'))
        except Exception as e:
            print(f"Erro ao enviar mensagem: {e}")
    
    def thread_receber_mensagens(self):
        """Thread para receber mensagens do servidor"""
        while self.conectado:
            try:
                dados = self.cliente_socket.recv(1024, socket.MSG_PEEK).decode('utf-8').strip()
                if not dados:
                    break
                
                if dados.startswith("CHAT_SERVER"):
                    dados = self.cliente_socket.recv(1024).decode('utf-8').strip()
                    # Mensagem de chat do servidor
                    mensagem = dados[12:]  # Remove "CHAT_SERVER "
                    print(f"\n[CHAT] {mensagem}")
                    print(">> ", end='', flush=True)
                
            except Exception as e:
                if self.conectado:
                    print(f"\nErro ao receber mensagem: {e}")
                break
    
    def sair(self):
        """Desconecta do servidor"""
        try:
            if self.conectado:
                self.cliente_socket.send("SAIR".encode('utf-8'))
                self.conectado = False
                self.cliente_socket.close()
                print("Desconectado do servidor.")
        except Exception as e:
            print(f"Erro ao desconectar: {e}")
    
    def mostrar_menu(self):
        """Mostra o menu de opções"""
        print("\n" + "="*50)
        print("COMANDOS DISPONÍVEIS:")
        print("SAIR                    - Desconectar e sair")
        print("ARQUIVO <nome>          - Baixar arquivo do servidor")
        print("CHAT <mensagem>         - Enviar mensagem no chat")
        print("HELP                    - Mostrar este menu")
        print("="*50)
    
    def executar(self):
        """Loop principal do cliente"""
        print("=== CLIENTE TCP ===")
        
        # Solicitar dados de conexão
        try:
            servidor_ip = input("Digite o IP do servidor (padrão: localhost): ").strip()
            if not servidor_ip:
                servidor_ip = "localhost"
            
            porta_str = input("Digite a porta do servidor (padrão: 8080): ").strip()
            if not porta_str:
                servidor_porta = 8080
            else:
                servidor_porta = int(porta_str)
        
        except KeyboardInterrupt:
            print("\nSaindo...")
            return
        except ValueError:
            print("Porta inválida!")
            return
        
        # Conectar ao servidor
        if not self.conectar(servidor_ip, servidor_porta):
            return
        
        # Iniciar thread para receber mensagens
        thread_receber = threading.Thread(target=self.thread_receber_mensagens, daemon=True)
        thread_receber.start()
        
        # Mostrar menu inicial
        self.mostrar_menu()
        
        # Loop principal de comandos
        try:
            while self.conectado:
                try:
                    comando = input(">> ").strip()
                    
                    if not comando:
                        continue
                    
                    partes = comando.split(' ', 1)
                    cmd = partes[0].upper()
                    
                    if cmd == "SAIR":
                        self.sair()
                        break
                    
                    elif cmd == "ARQUIVO":
                        if len(partes) < 2:
                            print("Uso: ARQUIVO <nome_do_arquivo>")
                            continue
                        
                        nome_arquivo = partes[1].strip()
                        if not nome_arquivo:
                            print("Nome do arquivo não pode estar vazio")
                            continue
                        
                        print(f"Solicitando arquivo: {nome_arquivo}")
                        self.receber_arquivo(nome_arquivo)
                    
                    elif cmd == "CHAT":
                        if len(partes) < 2:
                            print("Uso: CHAT <mensagem>")
                            continue
                        
                        mensagem = partes[1].strip()
                        if not mensagem:
                            print("Mensagem não pode estar vazia")
                            continue
                        
                        self.enviar_chat(mensagem)
                    
                    elif cmd == "HELP":
                        self.mostrar_menu()
                    
                    else:
                        print(f"Comando desconhecido: {cmd}")
                        print("Digite 'HELP' para ver os comandos disponíveis")
                
                except KeyboardInterrupt:
                    print("\nEncerrando...")
                    self.sair()
                    break
                
                except Exception as e:
                    print(f"Erro: {e}")
        
        except Exception as e:
            print(f"Erro no loop principal: {e}")
        
        finally:
            if self.conectado:
                self.sair()

if __name__ == "__main__":
    cliente = ClienteTCP()
    cliente.executar()
