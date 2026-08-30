---
title: Política de Privacidade
description: Como o Acerola trata dados locais, requisições a terceiros e conectividade entre dispositivos.
section: Privacidade
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import Card from '$lib/mdsvex/card.svelte';
	import CopyCard from '$lib/components/copy-card/copy-card.svelte';
</script>

<Callout type="note" title="Fonte de verdade">

Esta página espelha o [`PRIVACY_POLICY.md`](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/blob/main/PRIVACY_POLICY.md) na raiz do repositório. Em caso de divergência, o arquivo no repositório é a versão oficial.

</Callout>

<Card title="Controlador">Vinícius Gabriel Pereira Leitão, desenvolvedor independente do aplicativo Acerola.</Card>

<CopyCard
	label="Contato"
	value="contato@acerola-comic.com"
	href="mailto:contato@acerola-comic.com"
	copyLabel="Copiar"
	copiedLabel="Copiado"
/>

## 1. Natureza do aplicativo

Acerola é um leitor de arquivos CBZ/CBR que opera localmente no dispositivo do usuário. Não há criação de conta, login ou coleta de dados cadastrais.

## 2. Dados armazenados

Todos os dados gerados pelo uso do aplicativo (biblioteca, progresso de leitura, configurações) são armazenados exclusivamente no armazenamento local do dispositivo. Nenhum dado da biblioteca, progresso ou configurações é enviado a servidores próprios ou de terceiros para armazenamento — a única exceção envolve metadados de conectividade de rede quando a sincronização remota entre dispositivos está ativada (ver seção 3). A desinstalação do aplicativo remove permanentemente todos os dados armazenados localmente.

## 3. Conectividade e sincronização entre dispositivos

O Acerola sincroniza biblioteca, histórico e progresso de leitura diretamente entre os dispositivos do próprio usuário, sem passar por uma conta ou banco de dados central. Essa sincronização é opcional. Para como cada modo funciona tecnicamente, veja [Arquitetura](/docs/architecture) — aqui o foco é o que cada um implica em termos de dados:

<CardGrid>
	<Card title="Somente descoberta local">
		Padrão, gratuito. Nenhum dado de conectividade sai da rede local.
	</Card>
	<Card title="Relay público do iroh">
		O relay retransmite apenas tráfego QUIC criptografado ponta a ponta (TLS 1.3) — não acessa o conteúdo, mas os endereços IP dos dispositivos passam pela infraestrutura de terceiros do <a href="https://iroh.computer" target="_blank" rel="noopener noreferrer">iroh</a> durante a retransmissão.
	</Card>
	<Card title="Relay do próprio usuário">
		Rodando a própria instância do <code>acerola-relay</code> (MPL-2.0), nenhuma infraestrutura de terceiros ou do desenvolvedor do Acerola participa da conexão.
	</Card>
	<Card title="Relay hospedado pelo Acerola (pago)">
		Conteúdo continua criptografado ponta a ponta; apenas metadados de conexão (IPs, horário, volume de tráfego) passam pela infraestrutura do desenvolvedor. Dados de pagamento são tratados pelo provedor de pagamentos usado na contratação.
	</Card>
</CardGrid>

Em nenhum dos quatro modos o conteúdo da biblioteca (arquivos, progresso, histórico) é armazenado em servidores de relay — o relay atua só como intermediário de tráfego de rede.

<Callout type="note" title="Confira você mesmo">

Essa afirmação não pede confiança cega: o código do relay é aberto (MPL-2.0) e está em [`lib/relay/`](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/tree/main/lib/relay) no repositório — qualquer pessoa pode ler exatamente o que ele faz com o tráfego que passa por ele.

</Callout>

<Callout type="tip" title="Base legal">

O tratamento de metadados de conexão se baseia no legítimo interesse do controlador em viabilizar a sincronização solicitada pelo usuário (Art. 7º, IX, LGPD); no tier pago, a contratação em si se baseia no consentimento do usuário ao assinar o serviço (Art. 7º, I, LGPD).

</Callout>

## 4. Requisições a terceiros

Para busca de metadados e capas de obras, o aplicativo realiza requisições HTTP do tipo GET às APIs públicas do **MangaDex** e **AniList**. Essas requisições:

- não incluem dados pessoais do usuário (nome, e-mail, arquivos, etc.);
- contêm apenas os parâmetros de busca inseridos pelo usuário (ex: título da obra);
- resultam na exposição do endereço IP do dispositivo aos servidores desses serviços, como ocorre em qualquer requisição de rede.

O tratamento de dados nesse contexto se baseia no legítimo interesse do controlador em fornecer a funcionalidade solicitada pelo usuário (Art. 7º, IX, LGPD). Os dados retornados são usados apenas durante a execução do app e não são armazenados externamente por nós.

Políticas de privacidade dos terceiros: [MangaDex](https://mangadex.org/compliance/privacy) · [AniList](https://anilist.co/terms)

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
