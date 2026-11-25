-- Esquema corregido. Montos en CENTAVOS (INTEGER). Fechas en UTC, formato ISO 8601 con 'Z'.
-- Requiere SQLite >= 3.37 (tablas STRICT). Activar PRAGMA foreign_keys=ON en CADA conexión.

CREATE TABLE categorias (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre  TEXT NOT NULL UNIQUE,
    color   TEXT
) STRICT;

CREATE TABLE productos (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre         TEXT NOT NULL,
    barcode        TEXT UNIQUE,                 -- UNIQUE ya crea el índice
    precio_venta   INTEGER NOT NULL CHECK (precio_venta >= 0),
    precio_costo   INTEGER CHECK (precio_costo IS NULL OR precio_costo >= 0),
    stock          INTEGER NOT NULL DEFAULT 0,  -- se permite negativo (recuento atrasado)
    stock_minimo   INTEGER NOT NULL DEFAULT 5,
    activo         INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0,1)),
    categoria_id   INTEGER REFERENCES categorias(id),
    creado_en      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    actualizado_en TEXT
) STRICT;
CREATE INDEX idx_productos_nombre    ON productos(nombre);
CREATE INDEX idx_productos_categoria ON productos(categoria_id);

-- Cierres de caja (antes de ventas, porque ventas.cierre_id lo referencia)
CREATE TABLE cierres_caja (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha             TEXT NOT NULL UNIQUE,     -- día operativo local (YYYY-MM-DD): un cierre por día
    efectivo_inicial  INTEGER NOT NULL DEFAULT 0,
    efectivo_esperado INTEGER NOT NULL DEFAULT 0,
    efectivo_contado  INTEGER NOT NULL DEFAULT 0,
    diferencia_arqueo INTEGER NOT NULL DEFAULT 0, -- contado - esperado
    total_efectivo    INTEGER NOT NULL DEFAULT 0,
    total_mp          INTEGER NOT NULL DEFAULT 0,
    cantidad_ventas   INTEGER NOT NULL DEFAULT 0, -- cantidad, no monto
    monto_ventas      INTEGER NOT NULL DEFAULT 0, -- monto
    margen_bruto      INTEGER NOT NULL DEFAULT 0,
    delta_mp          INTEGER NOT NULL DEFAULT 0,
    creado_en         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

CREATE TABLE ventas (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    total          INTEGER NOT NULL CHECK (total >= 0),
    medio_pago     TEXT NOT NULL CHECK (medio_pago IN ('efectivo','mercado_pago','mixto')),
    monto_efectivo INTEGER NOT NULL DEFAULT 0 CHECK (monto_efectivo >= 0),
    monto_mp       INTEGER NOT NULL DEFAULT 0 CHECK (monto_mp >= 0),
    descripcion_mp TEXT,
    hora           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    cierre_id      INTEGER REFERENCES cierres_caja(id),  -- NULL = venta abierta
    CHECK (total = monto_efectivo + monto_mp)
) STRICT;
CREATE INDEX idx_ventas_hora   ON ventas(hora);
CREATE INDEX idx_ventas_cierre ON ventas(cierre_id);

CREATE TABLE venta_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id        INTEGER NOT NULL REFERENCES ventas(id),
    producto_id     INTEGER REFERENCES productos(id),    -- NULL = "cobro libre" sin producto
    descripcion     TEXT,                                 -- obligatoria si producto_id es NULL
    cantidad        INTEGER NOT NULL CHECK (cantidad > 0),
    precio_unitario INTEGER NOT NULL CHECK (precio_unitario >= 0),
    costo_unitario  INTEGER CHECK (costo_unitario IS NULL OR costo_unitario >= 0), -- costo al momento de la venta
    CHECK (producto_id IS NOT NULL OR descripcion IS NOT NULL)
) STRICT;
CREATE INDEX idx_venta_items_venta    ON venta_items(venta_id);
CREATE INDEX idx_venta_items_producto ON venta_items(producto_id);

CREATE TABLE devoluciones (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id        INTEGER NOT NULL REFERENCES ventas(id),
    medio_reintegro TEXT NOT NULL CHECK (medio_reintegro IN ('efectivo','mercado_pago','nota_credito')),
    motivo          TEXT,
    creado_en       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX idx_devoluciones_venta ON devoluciones(venta_id);

CREATE TABLE devolucion_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    devolucion_id   INTEGER NOT NULL REFERENCES devoluciones(id),
    venta_item_id   INTEGER NOT NULL REFERENCES venta_items(id), -- validar cantidad <= vendida - ya devuelta (en el use case)
    cantidad        INTEGER NOT NULL CHECK (cantidad > 0),
    monto_devuelto  INTEGER NOT NULL CHECK (monto_devuelto >= 0),
    reingresa_stock INTEGER NOT NULL DEFAULT 1 CHECK (reingresa_stock IN (0,1)) -- 0 = merma
) STRICT;
CREATE INDEX idx_devolucion_items_dev  ON devolucion_items(devolucion_id);
CREATE INDEX idx_devolucion_items_item ON devolucion_items(venta_item_id);

CREATE TABLE mp_pagos (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    mp_id            TEXT NOT NULL UNIQUE,             -- idempotencia de importación
    estado           TEXT NOT NULL CHECK (estado IN ('approved','refunded','charged_back','cancelled','other')),
    fecha_aprobacion TEXT,                             -- UTC
    monto_bruto      INTEGER NOT NULL,
    monto_neto       INTEGER,                          -- lo que realmente acredita MP (sin comisión)
    descripcion      TEXT,
    venta_id         INTEGER REFERENCES ventas(id),
    conciliado       INTEGER NOT NULL DEFAULT 0 CHECK (conciliado IN (0,1)),
    importado_en     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX idx_mp_pagos_fecha ON mp_pagos(fecha_aprobacion);
CREATE INDEX idx_mp_pagos_monto ON mp_pagos(monto_bruto);
CREATE INDEX idx_mp_pagos_venta ON mp_pagos(venta_id);

CREATE TABLE configuracion (
    clave TEXT PRIMARY KEY,
    valor TEXT
) STRICT;

-- Libro de movimientos: productos.stock debe ser igual a SUM(delta) por producto
CREATE TABLE movimientos_stock (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    producto_id    INTEGER NOT NULL REFERENCES productos(id),
    tipo           TEXT NOT NULL CHECK (tipo IN
                     ('venta','devolucion','reposicion','recuento_fisico','rotura','vencimiento','error_carga','otro')),
    delta          INTEGER NOT NULL CHECK (delta <> 0),
    stock_anterior INTEGER NOT NULL,
    stock_nuevo    INTEGER NOT NULL,
    venta_id       INTEGER REFERENCES ventas(id),
    devolucion_id  INTEGER REFERENCES devoluciones(id),
    operador       TEXT,
    creado_en      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    CHECK (stock_nuevo = stock_anterior + delta)
) STRICT;
CREATE INDEX idx_mov_stock_producto ON movimientos_stock(producto_id, creado_en);
