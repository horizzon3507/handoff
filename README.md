# Handoff

Envie arquivos entre computador e celular pela mesma rede Wi-Fi. Sem conta e sem nuvem.

## Tutorial rápido

### 1. Abra o Handoff no computador

Instale o [Node.js 22 ou mais recente](https://nodejs.org/) e o [Git](https://git-scm.com/). Na primeira vez, abra o terminal e rode estes comandos:

```sh
git clone https://github.com/horizzon3507/handoff.git
cd handoff
npm install
npm start
```

Nas próximas vezes, dentro da pasta `handoff`, basta iniciar o servidor:

```sh
npm start
```

Deixe o terminal e o computador ligados. No computador, abra <http://localhost:4173>. A página mostrará um QR code.

### 2. Conecte o celular

Conecte o celular ao mesmo Wi-Fi do computador. Abra a câmera, aponte para o QR code e toque no link que aparecer. Não digite `localhost` no celular: esse endereço só funciona no próprio computador.

### 3. Envie e baixe um arquivo

Para mandar uma foto do celular ao computador:

1. No celular, toque em **Choose files** e escolha a foto.
2. Aguarde ela aparecer em **Shared files**.
3. No computador, toque em **Download**. O navegador salva o arquivo na pasta de downloads.

Para mandar na direção contrária, escolha o arquivo no computador e toque em **Download** no celular. Os dois aparelhos podem enviar arquivos.

## Se o QR não funcionar

- Confira se os dois aparelhos estão na mesma rede Wi-Fi. Redes de convidados podem bloquear a conexão entre aparelhos.
- Mantenha o terminal aberto enquanto transfere.
- Se o Handoff foi fechado ou reiniciado, atualize a página no computador e escaneie o QR novo. O QR antigo expira com a sala.

## Encerrar

No terminal onde o Handoff está rodando, pressione `Ctrl+C`. A sala fecha e os arquivos temporários são apagados. Cada vez que abrir o Handoff, use o QR code novo.

## Limites e privacidade

Cada arquivo pode ter até 512 MB; cada sala aceita até 2 GB no total. Os arquivos ficam temporariamente no computador que está rodando o Handoff e não são enviados para uma nuvem.
