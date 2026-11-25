<script lang="ts">
	import { onMount } from 'svelte';
	import { verificarPin } from '$lib/api';
	import { appState } from '$lib/stores/app.svelte';

	let pin = $state('');
	let error = $state('');
	let verificando = $state(false);
	let inputPin: HTMLInputElement;

	onMount(() => {
		inputPin?.focus();
	});

	async function desbloquear(): Promise<void> {
		if (verificando || pin.length === 0) return;
		verificando = true;
		error = '';
		try {
			const correcto = await verificarPin(pin);
			if (correcto) {
				appState.bloqueado = false;
				pin = '';
				// Avisamos al layout para que reinicie el contador de inactividad.
				window.dispatchEvent(new Event('kiosco:actividad'));
			} else {
				error = 'PIN incorrecto';
				pin = '';
				inputPin?.focus();
			}
		} catch (e) {
			error = (e as Error).message;
			pin = '';
			inputPin?.focus();
		} finally {
			verificando = false;
		}
	}
</script>

<div class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/70 p-4">
	<div class="w-full max-w-sm rounded-2xl bg-white p-6 shadow-2xl">
		<h2 class="text-xl font-bold text-slate-900">Kiosco bloqueado</h2>
		<p class="mt-1 text-sm text-slate-500">
			Ingresá el PIN de 4 a 6 dígitos para volver a operar.
		</p>

		<form
			class="mt-5"
			onsubmit={(evento) => {
				evento.preventDefault();
				void desbloquear();
			}}
		>
			<label class="sr-only" for="pin-bloqueo">PIN</label>
			<input
				id="pin-bloqueo"
				bind:this={inputPin}
				bind:value={pin}
				type="password"
				inputmode="numeric"
				autocomplete="off"
				maxlength="6"
				placeholder="PIN"
				class="w-full rounded-lg border border-slate-300 px-3 py-2 text-center text-lg tracking-widest focus:border-blue-500 focus:outline-none"
			/>

			{#if error}
				<p class="mt-2 text-sm text-red-600" role="alert">{error}</p>
			{/if}

			<button
				type="submit"
				disabled={verificando || pin.length === 0}
				class="mt-4 w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{verificando ? 'Verificando…' : 'Desbloquear'}
			</button>
		</form>
	</div>
</div>
