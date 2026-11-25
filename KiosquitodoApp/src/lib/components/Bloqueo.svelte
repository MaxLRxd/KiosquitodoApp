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

<div class="fixed inset-0 z-40 flex items-center justify-center bg-slate-950/75 p-4 backdrop-blur-sm">
	<div class="modal-panel max-w-sm">
		<div class="grid h-11 w-11 place-items-center rounded-xl bg-gradient-to-br from-indigo-500 to-sky-500 text-lg font-black text-white shadow-lg shadow-indigo-950/30">
			K
		</div>
		<h2 class="modal-title mt-4">Kiosco bloqueado</h2>
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
				placeholder="••••"
				class="input py-3 text-center text-xl tracking-[0.5em]"
			/>

			{#if error}
				<p class="alert-error mt-3" role="alert">{error}</p>
			{/if}

			<button
				type="submit"
				disabled={verificando || pin.length === 0}
				class="btn-primary mt-4 w-full py-3"
			>
				{verificando ? 'Verificando…' : 'Desbloquear'}
			</button>
		</form>
	</div>
</div>
