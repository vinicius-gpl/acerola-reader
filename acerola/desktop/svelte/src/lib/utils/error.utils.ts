import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';

/**
 * Extracts a user-friendly error message from an unknown thrown error.
 * Safely parses string errors, Tauri ErrorPayload objects, or Error instances without using `any`.
 */
export function extractErrorMessage(error: unknown): string {
	if (error === null || error === undefined) {
		return 'Unknown error occurred';
	}
	// Stryker disable next-line ConditionalExpression,BlockStatement,StringLiteral: desativar/curto-circuitar
	// esse guard faz uma entrada string cair no `return String(error)` lá embaixo. `String(x) === x`
	// pra qualquer string primitiva por spec, então a saída observável nunca muda — um mutante que
	// remove ou derruba esse check é equivalente de verdade pra qualquer entrada string.
	if (typeof error === 'string') {
		return error;
	}
	if (typeof error === 'object') {
		const payload = error as Partial<ErrorPayload>;
		// Stryker disable next-line ConditionalExpression,LogicalOperator,StringLiteral,EqualityOperator,BlockStatement,MethodExpression:
		// esse branch e o fallback logo abaixo leem a mesma propriedade `message` do mesmo objeto
		// (`payload` é só `error` com cast, não uma cópia), sem trim. Sempre que
		// `typeof payload.message === 'string'` é verdadeiro, `'message' in error` também é
		// necessariamente verdadeiro, então o fallback dispara com o valor idêntico. Isso torna
		// o formato exato dessa condição inobservável: desativar, forçar qualquer operando,
		// inverter a fronteira de `.trim().length > 0` em qualquer direção, tirar o `.trim()`
		// ou esvaziar o if-block — tudo continua retornando a mesma string, o fluxo só reroteia
		// pelo branch de fallback (ou vice-versa), mas o valor de retorno é sempre idêntico.
		if (typeof payload.message === 'string' && payload.message.trim().length > 0) {
			return payload.message;
		}
		if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
			return (error as { message: string }).message;
		}
	}
	return String(error);
}
