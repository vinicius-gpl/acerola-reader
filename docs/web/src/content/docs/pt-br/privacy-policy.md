---
title: Política de Privacidade
description: Como o Acerola trata dados locais, requisições a terceiros e conectividade entre dispositivos.
section: Privacidade
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
</script>

<Callout type="note" title="Fonte de verdade">

Esta página espelha o [`PRIVACY_POLICY.md`](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/blob/main/PRIVACY_POLICY.md) na raiz do repositório. Em caso de divergência, o arquivo no repositório é a versão oficial.

</Callout>

**Controlador:** Vinícius Gabriel Pereira Leitão, desenvolvedor independente do aplicativo Acerola.
**Contato:** contato@acerola-comic.com

## 1. Natureza do aplicativo

Acerola é um leitor de arquivos CBZ/CBR que opera localmente no dispositivo do usuário. Não há criação de conta, login ou coleta de dados cadastrais.

## 2. Dados armazenados

Todos os dados gerados pelo uso do aplicativo (biblioteca, progresso de leitura, configurações) são armazenados exclusivamente no armazenamento local do dispositivo. Nenhum dado da biblioteca, progresso ou configurações é enviado a servidores próprios ou de terceiros para armazenamento — a única exceção envolve metadados de conectividade de rede quando a sincronização remota entre dispositivos está ativada (ver seção 3). A desinstalação do aplicativo remove permanentemente todos os dados armazenados localmente.

## 3. Conectividade e sincronização entre dispositivos

O Acerola sincroniza biblioteca, histórico e progresso de leitura diretamente entre os dispositivos do próprio usuário (ex.: desktop e Android), sem passar por uma conta ou banco de dados central. Essa sincronização é opcional e a forma como a conexão é estabelecida depende da configuração escolhida pelo usuário:

- **Somente descoberta local (padrão, gratuito)**: os dispositivos se encontram via mDNS na mesma rede local (Wi-Fi/LAN). Nenhum dado de conectividade sai da rede local.
- **Relay público do iroh**: quando os dispositivos não estão na mesma rede, o app pode usar a infraestrutura de relay pública do projeto [iroh](https://iroh.computer) para viabilizar a conexão (NAT traversal). O relay apenas retransmite tráfego QUIC criptografado ponta a ponta (TLS 1.3) entre os dispositivos — não tem acesso ao conteúdo da biblioteca, histórico ou progresso, mas, como em qualquer relay de rede, os endereços IP dos dispositivos conectados passam por essa infraestrutura de terceiros durante a retransmissão.
- **Relay do próprio usuário (self-hosted)**: o usuário pode rodar sua própria instância do `acerola-relay` (código aberto, MPL-2.0) em uma VPS própria. Nesse modo, nenhuma infraestrutura de terceiros ou do desenvolvedor do Acerola participa da conexão — o usuário é o único operador do relay.
- **Relay hospedado pelo Acerola (tier pago, opcional)**: alternativa paga em que o desenvolvedor do Acerola opera a infraestrutura de relay. Como nos demais modos, o conteúdo da biblioteca, histórico e progresso permanece criptografado ponta a ponta (TLS 1.3) e não é acessível ao relay — apenas metadados de conexão (endereços IP dos dispositivos, horário e volume de tráfego, necessários para rotear a conexão) passam pela infraestrutura do desenvolvedor enquanto a sincronização remota está em uso. Dados de pagamento da assinatura são tratados pelo provedor de pagamentos utilizado no momento da contratação — a política específica desse provedor será apresentada na tela de assinatura.

Em nenhum dos modos acima o conteúdo da biblioteca (arquivos, progresso, histórico) é armazenado em servidores de relay, sejam eles públicos, do usuário ou do desenvolvedor — o relay atua apenas como intermediário de tráfego de rede, nunca como local de armazenamento. O tratamento de metadados de conexão nesse contexto se baseia no legítimo interesse do controlador em viabilizar a funcionalidade de sincronização solicitada pelo usuário (Art. 7º, IX, LGPD); no caso do tier pago, a contratação em si se baseia no consentimento do usuário ao assinar o serviço (Art. 7º, I, LGPD).

## 4. Requisições a terceiros

Para busca de metadados e capas de obras, o aplicativo realiza requisições HTTP do tipo GET às APIs públicas do **MangaDex** e **AniList**. Essas requisições:

- não incluem dados pessoais do usuário (nome, e-mail, arquivos, etc.);
- contêm apenas os parâmetros de busca inseridos pelo usuário (ex: título da obra);
- resultam na exposição do endereço IP do dispositivo aos servidores desses serviços, como ocorre em qualquer requisição de rede.

O tratamento de dados nesse contexto se baseia no legítimo interesse do controlador em fornecer a funcionalidade solicitada pelo usuário (Art. 7º, IX, LGPD). Os dados retornados são usados apenas durante a execução do app e não são armazenados externamente por nós.

Políticas de privacidade dos terceiros:

- MangaDex: <https://mangadex.org/compliance/privacy>
- AniList: <https://anilist.co/terms>

## 5. Compartilhamento de dados

O Acerola não compartilha, vende ou transmite dados a terceiros além das requisições descritas no item 4 (MangaDex/AniList) e do trânsito de metadados de conectividade descrito no item 3 (relay), ambos estritamente necessários ao funcionamento das respectivas funcionalidades.

## 6. Menores de idade

O aplicativo não coleta dados pessoais de nenhum usuário, independentemente da idade, e não é direcionado especificamente a menores de 18 anos.

## 7. Direitos do titular (Art. 18, LGPD)

Como nenhum dado pessoal é armazenado por nós, o exercício de direitos como acesso, correção e eliminação se dá diretamente pelo usuário, localmente, ao gerenciar ou desinstalar o aplicativo, ou desativando a sincronização remota (item 3). Dúvidas sobre dados eventualmente processados por MangaDex, AniList, pela infraestrutura de relay do iroh ou pelo provedor de pagamentos devem ser dirigidas a esses serviços.

## 8. Segurança

Ainda que não haja coleta de dados pessoais, o aplicativo adota práticas de desenvolvimento seguro para garantir a integridade de seu funcionamento, incluindo o uso de criptografia ponta a ponta (TLS 1.3) em toda a sincronização entre dispositivos, independentemente do modo de conectividade escolhido.

## 9. Lei aplicável

Esta política é regida pela legislação brasileira, incluindo a Lei nº 13.709/2018 (LGPD).
