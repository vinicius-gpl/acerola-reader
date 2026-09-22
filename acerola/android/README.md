# acerola-android

Acerola é um leitor de mangá para Android focado em coleções locais. Basta apontar para uma pasta no dispositivo que o aplicativo encontra os arquivos, organiza automaticamente sua biblioteca e deixa tudo pronto para leitura.

---

## Preview

<p align="center">
  <img src="../../docs/github/android/banner/acerola-android-solo.png" alt="Acerola Android" width="70%">
</p>

---

## Galeria

<table>
  <tr>
    <td rowspan="3" valign="top" align="center">
      <img src="../../docs/github/android/prints/reader-screen.png" width="260" style="display:block;"><br>
      <sub><b>Leitura</b></sub>
    </td>
    <td align="center">
      <img src="../../docs/github/android/prints/home-screen.png" width="140"><br>
      <sub><b>Home</b></sub>
    </td>
    <td align="center">
      <img src="../../docs/github/android/prints/chapters-screen.png" width="140"><br>
      <sub><b>Capítulos</b></sub>
    </td>
    <td align="center">
      <img src="../../docs/github/android/prints/config-screen.png" width="140"><br>
      <sub><b>Configurações</b></sub>
    </td>
  </tr>

  <tr>
    <td align="center">
      <img src="../../docs/github/android/prints/history-screen.png" width="140"><br>
      <sub><b>Histórico</b></sub>
    </td>
    <td align="center">
      <img src="../../docs/github/android/prints/network-screen.png" width="140"><br>
      <sub><b>Rede</b></sub>
    </td>
    <td></td>
  </tr>
</table>

---

```mermaid
flowchart LR
    Pasta[Sua pasta<br/>no dispositivo] --> Scan[Acerola escaneia]
    Scan --> Biblioteca[Biblioteca organizada]
    Biblioteca --> Metadados[Busca capas e<br/>metadados]
    Biblioteca --> Leitura[Comece a ler]
```

---

## Funcionalidades

* **Biblioteca**

  * Escaneia automaticamente uma ou mais pastas.
  * Detecta novos arquivos.
  * Visualização em grade ou lista.
  * Busca rápida.
  * Organização por categorias.

* **Metadados**

  * Busca automaticamente capa, sinopse, autor e gênero.
  * Integração com MangaDex, AniList e ComicInfo.
  * Alteração manual das informações.
  * Troca da fonte de metadados quando desejar.

* **Leitura**

  * Suporte nativo para `.cbz` e `.cbr`.
  * Conversão automática de `.pdf` para `.cbz`.
  * Diversos modos de leitura.
  * Paginação configurável.
  * Salvamento automático do progresso.

* **Histórico**

  * Acesso rápido aos mangás lidos recentemente.

* **Personalização**

  * Temas Catppuccin.
  * Dracula.
  * Alucard.
  * Nord.
  * Outras opções de customização da interface.

---

## Como funciona

```mermaid
flowchart TD
    A[Abrir o aplicativo] --> B[Conceder permissão de armazenamento]
    B --> C[Selecionar a pasta da coleção]
    C --> D[Escaneamento automático]
    D --> E[Biblioteca pronta]

    E --> F{Próximo passo}

    F --> G[Sincronizar metadados]
    F --> H[Abrir um mangá]

    H --> I[Escolher um capítulo]
    I --> J[Começar a leitura]
```

1. Abra o aplicativo.
2. Conceda a permissão de armazenamento.
3. Escolha a pasta onde sua coleção está armazenada.
4. Aguarde o escaneamento automático.
5. Sincronize os metadados para baixar capas e informações.
6. Abra um mangá e comece a leitura.

---

## Formatos suportados

| Formato | Descrição                                                  |
| ------- | ---------------------------------------------------------- |
| `.cbz`  | Comic Book ZIP (ZIP contendo imagens)                      |
| `.cbr`  | Comic Book RAR (RAR contendo imagens)                      |
| `.pdf`  | Convertido automaticamente para `.cbz` na primeira leitura |

---

## Futuro do ecossistema

### acerola-desktop

O Acerola para Android fará parte do ecossistema **acerola-desktop**, permitindo que a biblioteca seja compartilhada entre computador e celular.

Os recursos planejados incluem:

* Biblioteca do computador acessível diretamente pelo Android.
* Sincronização de progresso de leitura.
* Sincronização de histórico.
* Envio de mangás entre dispositivos.
* Descoberta automática na mesma rede.
* Acesso remoto utilizando **acerola-relay**, sem necessidade de abrir portas no roteador ou configurar VPN.
