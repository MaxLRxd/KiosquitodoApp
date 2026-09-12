import json
from graphify.build import build_from_json
from graphify.cluster import score_all
from graphify.analyze import god_nodes, surprising_connections, suggest_questions
from graphify.report import generate
from graphify.export import to_json
from pathlib import Path

extraction = json.loads(Path('graphify-out/.graphify_extract.json').read_text(encoding='utf-8'))
detection = json.loads(Path('graphify-out/.graphify_detect.json').read_text(encoding='utf-8'))
analysis = json.loads(Path('graphify-out/.graphify_analysis.json').read_text(encoding='utf-8'))

G = build_from_json(extraction, root='.', directed=False)
communities = {int(k): v for k, v in analysis['communities'].items()}
cohesion = {int(k): v for k, v in analysis['cohesion'].items()}
tokens = {'input': extraction.get('input_tokens', 0), 'output': extraction.get('output_tokens', 0)}

labels = {
    0: "Conciliation Matching Service",
    1: "Common App Types & Errors",
    2: "Venta Pricing & Margins",
    3: "Product Service Tests",
    4: "Cierre de Caja Service",
    5: "Venta Service Mocks",
    6: "Cierre SQLite Persistence",
    7: "Product SQLite Persistence",
    8: "Keyring Credentials Store",
    9: "Frontend Package Metadata",
    10: "Docs: Tauri Commands & Pages",
    11: "Tauri App Configuration",
    12: "Docs: Architecture & Rationale",
    13: "Frontend Dev Dependencies",
    14: "Config Module Resolution",
    15: "Conciliation Domain Logic",
    16: "Product Repository Ports",
    17: "Docs: Scanner Input Design",
    18: "Docs: MercadoPago Reconciliation",
    19: "Docs: Infrastructure Conventions",
    20: "Toast & App State",
    21: "TypeScript Compiler Config",
    22: "Payment Conversion Logic",
    23: "SQLite Migrations",
    24: "Frontend Scripts",
    25: "Backup Management",
    26: "Learning Skill Docs",
    27: "Docs: Scanner Hardware Integration",
    28: "System Infrastructure Modules",
    29: "SvelteKit App Types",
    30: "Frontend Domain Types",
    31: "Project Docs Spec & README",
    32: "Autostart Module",
    33: "Scanner Listener",
    34: "Svelte Config",
    35: "App Icons & Favicon",
    36: "Logging Init",
    37: "Sound Effects",
    38: "Tauri Runtime Deps",
    39: "API Client",
    40: "Date Formatting",
    41: "Currency Formatting",
    42: "SSR Layout",
    43: "Build Script",
    44: "Lib Run Entry",
    45: "Main Entry",
    46: "Tailwind Config",
    47: "App State Import",
    48: "PostCSS Config",
    49: "Cierre Page",
    50: "Conciliacion Page",
    51: "Configuracion Page",
    52: "Devoluciones Page",
    53: "Inventario Page",
    54: "Home Page",
    55: "Application Module",
    56: "Domain Module",
    57: "API Module",
    58: "MercadoPago Module",
    59: "Infrastructure Module",
    60: "System Module",
    61: "Setup Script",
    62: "Vite Config",
    63: "Kiosco App Root",
}

questions = suggest_questions(G, communities, labels)

report = generate(G, communities, cohesion, labels, analysis['gods'], analysis['surprises'], detection, tokens, '.', suggested_questions=questions)
Path('graphify-out/GRAPH_REPORT.md').write_text(report, encoding='utf-8')
Path('graphify-out/.graphify_labels.json').write_text(json.dumps({str(k): v for k, v in labels.items()}, ensure_ascii=False), encoding='utf-8')
wrote = to_json(G, communities, 'graphify-out/graph.json', community_labels=labels)
if not wrote:
    print('ERROR: refused to shrink graphify-out/graph.json (existing graph has more nodes; #479).')
print('Report updated with community labels')