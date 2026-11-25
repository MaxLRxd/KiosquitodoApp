<div class="page">
	<div class="page-header">
		<div>
			<p class="eyebrow">Mercado Pago</p>
			<h1 class="page-title mt-1">Conciliación</h1>
			<p class="page-subtitle">
				Ventas MP registradas contra pagos acreditados según la API. Módulo en construcción (M6).
			</p>
		</div>
		<span class="badge-warn">En construcción</span>
	</div>

	<div class="card card-pad mt-5">
		<div class="flex flex-col items-start gap-4 sm:flex-row sm:items-center">
			<div class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl bg-sky-50 text-sky-600">
				<svg
					class="h-6 w-6"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.7"
					stroke="currentColor"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="M7.5 7.5h12m0 0-3-3m3 3-3 3" />
					<path d="M16.5 16.5h-12m0 0 3 3m-3-3 3-3" />
				</svg>
			</div>
			<div>
				<h2 class="section-title">La conciliación llega en el próximo módulo</h2>
				<p class="section-subtitle">
					Requiere que los pagos se sincronicen desde Mercado Pago. Mientras tanto, el token se
					administra desde <span class="font-semibold text-slate-700">Configuración</span> y el
					delta MP ya aparece en <span class="font-semibold text-slate-700">Cierre de caja</span>.
				</p>
			</div>
		</div>

		<div class="mt-5 grid gap-3 sm:grid-cols-3">
			<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
				<p class="stat-label">Paso 1</p>
				<p class="mt-1 text-sm font-semibold text-slate-800">Sincronizar pagos</p>
				<p class="mt-1 text-xs text-slate-500">Traer los pagos acreditados del período.</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
				<p class="stat-label">Paso 2</p>
				<p class="mt-1 text-sm font-semibold text-slate-800">Vincular ventas</p>
				<p class="mt-1 text-xs text-slate-500">Emparejar cada pago con su venta MP.</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
				<p class="stat-label">Paso 3</p>
				<p class="mt-1 text-sm font-semibold text-slate-800">Revisar diferencias</p>
				<p class="mt-1 text-xs text-slate-500">Detectar huérfanos y deltas por día.</p>
			</div>
		</div>
	</div>
</div>
