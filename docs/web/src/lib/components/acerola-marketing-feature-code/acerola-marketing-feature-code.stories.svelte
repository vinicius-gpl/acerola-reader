<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import ZapIcon from '@lucide/svelte/icons/zap';
	import AcerolaMarketingFeatureCode from './acerola-marketing-feature-code.svelte';

	const { Story } = defineMeta({
		component: AcerolaMarketingFeatureCode,
		title: 'Componentes/AcerolaMarketingFeatureCode',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Seção de marketing com divisão entre lista de recursos e painel de código interativo com suporte a abas e cópia, inspirado no Cult UI.'
				}
			}
		}
	});
</script>

{#snippet template()}
	<div class="mx-auto max-w-5xl p-6">
		<AcerolaMarketingFeatureCode
			eyebrow="Arquitetura P2P"
			title="Sincronização Direta sem Nuvem"
			description="Conectividade de alta performance entre desktop e mobile usando Iroh e QUIC."
			features={[
				{
					icon: WaypointsIcon,
					title: 'Direct QUIC Streams',
					description: 'Transferência binária em ponto a ponto com multiplexação.'
				},
				{
					icon: ShieldCheckIcon,
					title: 'Criptografia de Ponta a Ponta',
					description: 'Autenticação por chave pública Ed25519 e verificação BLAKE3.'
				},
				{
					icon: ZapIcon,
					title: 'Relay Fallback Automático',
					description: 'Atravessa NAT e firewalls através de servidor relay dedicado.'
				}
			]}
			tabs={[
				{
					id: 'p2p',
					label: 'p2p-sync.rs',
					code: `// Conexão direta via Iroh endpoint\nlet endpoint = Endpoint::builder()\n    .secret_key(secret_key)\n    .alpns(vec![ALPN_ACEROLA_SYNC])\n    .bind()\n    .await?;\n\n// Handshake e sincronização de arquivo CBZ\nlet conn = endpoint.connect(peer_addr, ALPN_ACEROLA_SYNC).await?;\nlet (mut send, mut recv) = conn.open_bi().await?;`
				},
				{
					id: 'caddy',
					label: 'acerola-relay.caddy',
					code: `relay.{$DOMAIN} {\n    reverse_proxy acerola-relay:80 {\n        header_up Host {host}\n        header_up X-Real-IP {remote}\n    }\n    encode zstd gzip\n}`
				}
			]}
		/>
	</div>
{/snippet}

<Story name="Default" {template} />
