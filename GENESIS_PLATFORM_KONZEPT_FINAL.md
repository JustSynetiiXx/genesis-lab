# Genesis Lab — Konzeptdokument

## Stand: 24. März 2026
## Status: Alle Entscheidungen getroffen — bereit zum Bau

---

## 1. Was ist es?

Eine Web-Plattform auf der jeder seine eigene digitale Evolution starten kann. Der User definiert die Physik seiner Welt — Nahrung, Mutation, Verfall, Speicher — drückt Start und beobachtet was emergiert. Keine vorgefertigten Ergebnisse, keine Skripte. Echte Evolution, live im Browser.

Im Kern: Genesis v3 als Produkt. Aber vereinfacht, interaktiv, zugänglich.

---

## 2. Warum kauft jemand das?

### Die Zielgruppe

- **Simulations-Nerds:** Leute die Factorio, Oxygen Not Included, The Bibites, Species: ALRE spielen. Sie lieben Systeme, Parameter, emergentes Verhalten.
- **Studenten & Hobbyisten:** Bio, Informatik, Komplexitätstheorie. Die wollen Evolution verstehen, nicht nur darüber lesen.
- **ALife-Community:** Forscher und Enthusiasten die experimentieren wollen ohne selbst Code zu schreiben.

### Warum sie zahlen

Nicht für das Zuschauen. Für das **Experimentieren**. Der Reiz ist: "Ich habe die Parameter so eingestellt, dass Aggression emergiert." Oder: "In meiner Welt leben Organismen mit 20 Bytes länger als 3 Stunden." Das ist kompetitiv, kreativ, und endlos.

### Warum kein Konkurrent das hat

- **Species: ALRE** — simuliert Evolution, aber geskriptet. Neuronale Netze + Fitness-Funktion.
- **The Bibites** — ähnlich, Fitness-basiert.
- **Tierra** — echte Evolution, aber keine Wahrnehmung, kein Dashboard, kein Produkt.
- **Genesis Lab** — echte Evolution MIT Wahrnehmung, MIT Aggression, MIT Nahrung. Publiziert, nachgewiesen. Kein anderes Produkt auf dem Markt bietet das.

---

## 3. Was kann der User?

### Kostenlos (Free Tier)

- Eine Simulation starten mit Standard-Parametern
- Kleiner Speicher (256 KB)
- Live-Dashboard: Population, Diversität, Weltkarte
- Simulation läuft solange der Browser offen ist
- Limitiert auf 1 Welt gleichzeitig

### Bezahlt (7€/Monat)

- **Parameter-Editor:** Alle Stellschrauben offen
  - Speichergröße (bis 10 MB)
  - Nahrungsmenge und -verteilung
  - Mutationsrate
  - Verfallsrate
  - Blitz-Häufigkeit
  - Kopier-Kosten
  - Energie-Cap
- **Persistente Welten:** Simulation läuft weiter wenn der Browser zu ist (Server-seitig)
- **Mehrere Welten:** Bis zu 5 parallele Simulationen
- **Volles Dashboard:** Analyse-Tab, Genom-Viewer, Operations-Verteilung
- **Datenexport:** JSON-Snapshots, CSV
- **Zeitraffer:** Simulation beschleunigen

### Möglich in Zukunft (nicht zum Launch)

- Leaderboard: "Längste überlebende Population", "Meiste Arten", "Kleinster Organismus"
- Welt teilen: Öffentliche Welten die andere beobachten können
- Presets: Von anderen Usern erstellte Parameter-Sets
- Community-Challenges: "Erstelle eine Welt in der Aggression dominiert"

---

## 4. Architektur

### Prinzip

Zwei Modi: Free-User simulieren lokal im Browser (WebAssembly). Paid-User simulieren auf dem Server (Rust nativ). Das Frontend ist identisch — nur die Datenquelle wechselt.

```
FREE TIER:
┌──────────────────────────────────────────────┐
│  BROWSER                                     │
│  Frontend (Dashboard, Parameter-Editor)      │
│  Genesis-Kern als WebAssembly (WASM)         │
│  Beobachter läuft auch im Browser            │
│  Alles lokal — kein Server nötig             │
└──────────────────────────────────────────────┘

PAID TIER:
┌──────────────────────────────────────────────┐
│  BROWSER                                     │
│  Frontend (Dashboard, Parameter-Editor)      │
│  Kommuniziert über WebSocket mit Backend     │
├──────────────────────────────────────────────┤
│  BACKEND (VPS)                               │
│  API-Server — User-Management, Auth, Billing │
│  Simulations-Manager — startet/stoppt Welten │
│  WebSocket-Server — Live-Daten zum Frontend  │
├──────────────────────────────────────────────┤
│  SIMULATIONEN (isoliert pro User)            │
│  Rust-Interpreter nativ, ein Prozess pro Sim │
│  Beobachter-Daten → WebSocket → Frontend     │
│  Läuft 24/7, auch wenn User offline ist      │
└──────────────────────────────────────────────┘
```

### Technologie

- **Simulations-Kern:** Rust (kompiliert nativ für Server + zu WebAssembly für Browser)
- **Backend:** Python (FastAPI) oder Node.js — API, Auth, Simulations-Manager
- **Frontend:** React oder HTML + JS — Dashboard, Parameter-Editor, WASM-Integration
- **Datenbank:** SQLite oder PostgreSQL für User-Daten, Parameter, Snapshots
- **Auth:** Einfach — E-Mail + Passwort, oder GitHub OAuth
- **Zahlung:** Stripe (einfachste Integration, keine Bürokratie)
- **Hosting:** Eigener VPS (Hetzner, Netcup — ab 5€/Monat pro Server)

### Performance — ENTSCHIEDEN: Rust von Anfang an

Genesis auf PyPy3 schafft ~66 Ticks/s. Für eine Multi-User-Plattform reicht das nicht. Deshalb wird der Interpreter von Anfang an in Rust geschrieben. Kein Python-Port, kein "später mal".

**Was Rust bringt:**

- **100-1000x schneller als PyPy3.** Konservativ geschätzt 5.000-50.000 Ticks/s pro Simulation. Ein VPS mit 4 Kernen könnte 100+ Simulationen parallel laufen lassen.
- **WebAssembly (WASM).** Rust kompiliert zu WASM. Das heißt: Die Free-Tier-Simulation läuft direkt im Browser des Users — kein Server nötig. Der User öffnet die Seite, die Simulation startet lokal auf seinem Gerät. Das spart massiv Server-Kosten.
- **PyO3-Anbindung.** Falls nötig kann der Rust-Kern über PyO3 auch von Python aus aufgerufen werden — z.B. für das bestehende Genesis-Dashboard oder den Beobachter.
- **Sicherheit.** Rusts Speicher-Modell verhindert Buffer Overflows und Race Conditions. Bei einer Plattform mit User-definierten Parametern ist das relevant.

**Architektur-Entscheidung daraus:**

| Tier | Wo läuft die Simulation? | Wie? |
|------|--------------------------|------|
| Free | Im Browser des Users | Rust → WebAssembly (WASM) |
| Paid | Auf dem Server | Rust nativ, als eigener Prozess pro User |

**Free Tier im Browser bedeutet:**
- Kein Server-Last für kostenlose User
- Simulation stoppt wenn der Browser zu ist (das ist die Einschränkung, kein Bug)
- Keine Kosten pro Free-User für Max

**Paid Tier auf dem Server bedeutet:**
- Simulation läuft 24/7 weiter, auch wenn der User offline ist
- Max kontrolliert die Infrastruktur
- Datenexport und Snapshots server-seitig

**Ergänzende Maßnahmen (zusätzlich zu Rust):**
- Zeitscheiben: Inaktive Paid-Welten bekommen weniger CPU-Zeit
- Kleinere Free-Welten: 256 KB statt 1 MB reduziert WASM-Speicherbedarf im Browser
- Scale on demand: Mehr VPS-Kapazität wenn mehr zahlende User kommen

---

## 5. Was Max baut vs. was die Agents machen

### Max + Claude Code baut:

- Den Rust-Interpreter (Port von Python, Woche 1-2)
- Die WASM-Integration für den Browser
- Die Plattform (Backend + Frontend)
- Das Dashboard
- Die Bezahl-Integration (Stripe)
- Die Landing Page

### Agents / Automatisierung übernimmt:

- **Support:** FAQ-Bot auf der Webseite, keine E-Mails
- **Onboarding:** Automatische Willkommens-Mail, Tutorial im Tool
- **Monitoring:** Alert wenn ein Server voll ist oder eine Simulation crasht
- **Billing:** Stripe managed alles — Abo, Kündigung, Rechnungen

### Max macht NICHT:

- Telefonate
- Kunden-E-Mails beantworten
- Social Media pflegen
- Community managen

---

## 6. Zeitplan (realistisch)

### Woche 1-2: WASM-Build + Minimales Frontend
- Rust-Core für WebAssembly kompilieren (wasm-pack, wasm-bindgen)
- JavaScript-Bridge: Parameter rein → Simulation starten → Daten raus
- Minimales Frontend: Parameter wählen → Start → Live-Zahlen sehen
- Dark Mode, Dashboard-Grundgerüst

### Woche 3-4: Volles Dashboard + Landing Page
- Weltkarte, Population-Graph, Operations-Verteilung
- Parameter-Editor im Browser (alle Stellschrauben)
- Landing Page (was ist Genesis Lab, Free vs. Paid, Paper-Link)
- Genom-Viewer, Analyse-Tab

### Woche 5-6: Backend + Paid Tier
- API-Server für User-Accounts und Stripe-Billing (7€/Monat)
- Server-seitige Simulation (Rust nativ) für zahlende User
- WebSocket-Verbindung für Live-Daten
- Multi-Tenant: Isolierte Simulationen pro User

### Woche 7-8: Launch
- Free Tier (WASM) + Paid Tier (Server) beide funktionsfähig
- Deployment auf VPS (Hetzner/Netcup)
- Tests, Bugfixes, Polish
- **Ein Reddit-Post:** r/artificial, r/compsci, r/IndieGaming, r/cellular_automata

### Nach Launch:
- Daten sammeln (User-Verhalten, populäre Parameter)
- Features basierend auf Nutzung
- Leaderboard, Welt-Teilen, Challenges

---

## 7. Kosten

| Posten | Kosten |
|--------|--------|
| VPS für Plattform (Hetzner CX31 oder ähnlich) | ~10€/Monat |
| Domain | ~10€/Jahr |
| Stripe | 1,4% + 0,25€ pro Transaktion |
| Claude Pro Max (bereits vorhanden) | 0€ zusätzlich |
| **Gesamt zum Start** | **~15€/Monat** |

---

## 8. Einnahmen-Szenario (konservativ)

| Monat | Zahlende User | Einnahmen (bei 7€/Monat) |
|-------|---------------|--------------------------|
| 1 | 5-10 | 35-70€ |
| 3 | 20-50 | 140-350€ |
| 6 | 50-100 | 350-700€ |
| 12 | 100-300 | 700-2.100€ |

**Ehrlich:** Das ist kein Ersatz für deinen Job in Monat 1. Aber es ist passives Einkommen das wächst. Und es ist DEIN Projekt, DEIN System, DEINE Forschung.

---

## 9. Der Forschungs-Aspekt

### Was die Plattform für dich als Forscher bedeutet

Jeder User der eine Simulation startet exploriert den Parameterraum für dich. Tausende Konfigurationen die du alleine nie testen könntest. Anonymisierte Daten — welche Parameter produzieren Aggression? Bei welcher Mutationsrate emergiert Kooperation? Wie beeinflusst die Speichergröße die Diversität?

Das ist ein verteiltes Forschungslabor, finanziert durch die User.

### Nutzungsbedingungen

- Alle Simulationsdaten dürfen anonymisiert für Forschungszwecke verwendet werden
- Entdeckungen auf der Plattform werden im Paper credited: "Entdeckt auf Genesis Lab, entwickelt von Maximilian Seiler"
- Die Plattform selbst ist das geistige Eigentum von Max
- Der Genesis-Forschungskern bleibt Open Source (wie bisher)

### Dein Genesis auf dem VPS

Bleibt unangetastet. Läuft weiter mit deinen Parametern, deinen Daten. Die Plattform ist ein separates Produkt, basierend auf demselben Kern. Das eine entwertet das andere nicht.

---

## 10. Entscheidungen (24. März 2026)

| Frage | Entscheidung |
|-------|-------------|
| Name | **Genesis Lab** |
| Pricing | **7€/Monat** Abo |
| Design | **Dark Mode** (wie Genesis-Dashboard) |
| Repo | **Neue Repo: genesis-lab** (getrennt von Genesis-V3) |
| Rust-Core | **Bereits fertig** — 325 Ticks/s auf VPS, alle 11 Opcodes, 2D-Topologie, Gradienten, Typ-B-Nahrung |
| Free Tier | 256 KB Speicher, läuft im Browser (WASM), stoppt bei Browser-Close |
| Forschungs-Genesis | **Läuft bereits auf Rust-Core** auf dem VPS |

### Aktueller technischer Stand (Genesis-V3 Repo)

Was bereits existiert und für Genesis Lab wiederverwendet wird:
- Rust-Interpreter mit allen 11 Opcodes (`genesis-core/src/interpreter.rs`)
- Welt mit Nahrung, Verfall, Blitze, 2D-Grid-Topologie (`genesis-core/src/welt.rs`)
- Konfigurierbares Parameter-System über CLI (`genesis-core/src/config.rs`)
- HTTP-API mit Export, Status, Weltkarte, Analyse, Trace (`genesis-core/src/http_api.rs`)
- Typ-B-Nahrung als Kopier-Voraussetzung (neue Mechanik)
- Nahrungsgradienten mit Oasen-System
- Abiogenese (Auto-Respawn bei Pop=0)

### Was für Genesis Lab NEU gebaut werden muss

1. **WASM-Build** des Rust-Kerns für Browser-Simulation (Free Tier)
2. **Frontend** — Dashboard, Parameter-Editor, Landing Page (Dark Mode)
3. **Backend** — User-Accounts, Stripe-Billing, Simulations-Manager
4. **Multi-Tenant** — Mehrere isolierte Simulationen pro Server
5. **WebSocket** — Live-Daten vom Server zum Browser (Paid Tier)

---

## 11. Risiken (ehrlich)

- **Kein User zahlt:** Möglich. Deshalb Free Tier — erstmal Nutzer gewinnen, dann konvertieren.
- **WASM-Build funktioniert nicht reibungslos:** Der Rust-Core muss für WebAssembly kompiliert werden. Manche Abhängigkeiten (tiny_http, rand mit OS-Features) funktionieren nicht in WASM. Lösung: WASM-spezifisches Feature-Set im Cargo.toml.
- **WASM-Performance im Browser reicht nicht:** Auf schwachen Geräten (altes Handy, Tablet) könnte die Simulation ruckeln. Lösung: Tick-Rate limitieren, 256 KB statt 1 MB für Free Tier.
- **Keiner findet die Plattform:** Deshalb Reddit-Post. Wenn der floppt, Hacker News, ALife-Foren, Paper-Zitation.
- **Jemand klont es:** Genesis-Kern ist Open Source. Aber dein Branding, dein Paper, dein Name sind nicht klonbar. Du bist der Forscher, nicht der Kopierer.
- **Zu wenig Zeit:** Du baust am Handy, abends, mit Claude Code. Genau wie Genesis. Das hat funktioniert.

---

*"Wir lehren Genesis nichts. Wir bauen den Raum."*
*Jetzt bauen wir den Raum für andere.*

---

## 12. Erster Claude Code Prompt

Neues Repo `genesis-lab` erstellen. Erster Schritt: Den bestehenden Rust-Core aus Genesis-V3 für WebAssembly kompilierbar machen. Kein Server-Code (tiny_http), kein OS-abhängiger RNG — nur der reine Interpreter + Welt als WASM-Modul mit JavaScript-API.
