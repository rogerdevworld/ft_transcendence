### 1. Mapa de Módulos Seleccionados (19 puntos)

* **Web (9 pts):**
* **Major (2 pts):** Frameworks en Frontend (React) y Backend (Axum en Rust).


* **Major (2 pts):** Tiempo real con WebSockets (`axum::extract::ws` + Redis Pub/Sub) para votos en vivo, playlists y delegación de control.


* **Major (2 pts):** Interacción entre usuarios (chat en salas, perfiles y sistema de amigos).


* **Minor (1 pt):** ORM para base de datos (**SeaORM** sobre PostgreSQL con migraciones `sea-orm-migration`).


* **Minor (1 pt):** Funciones colaborativas en tiempo real (edición multiusuario simultánea de playlists con control de concurrencia).


* **Minor (1 pt):** Búsqueda avanzada (filtros por género/visibilidad, ordenamiento por votos y paginación en API).




* **User Management (3 pts):**
* **Major (2 pts):** Gestión estándar de usuarios y autenticación (registro, perfiles público/amigos/privado, avatar con default y estado online).


* **Minor (1 pt):** Autenticación remota con OAuth 2.0 (Google / 42 usando `oauth2` crate).




* **Artificial Intelligence (5 pts):**
* **Major (2 pts):** Sistema de recomendación ML (filtrado colaborativo/basado en contenido sobre preferencias, likes e historial en Rust o vía servicio/modelo embebido).


* **Major (2 pts):** Interfaz completa de LLM (asistente musical/generador de playlists en *streaming* SSE/WebSockets con manejo de errores y *rate limiting* vía `tower_governor`).


* **Minor (1 pt):** IA de moderación de contenido (filtro automático de mensajes en chat y nombres de salas).




* **Cybersecurity (2 pts):**
* **Major (2 pts):** WAF/ModSecurity (hardened) en el reverse proxy Nginx + HashiCorp Vault (`vaultrs` crate) para gestión cifrada de secretos.





(Nota para evaluación 42: aunque la programación queda dividida estrictamente entre **Roger** y **Alejandro**, en el `README.md` y en el historial de Git deben figurar los 4 integrantes —Roger, Alejandro, Ayub y Persona Cuarta— con los roles requeridos —PO, PM, Tech Lead, Developers— y commits reales, dejando a Ayub y Persona Cuarta QA, testing, documentación y páginas legales).

---

### 2. Estructura de Carpetas del Monorepo (Rust + React)

```text
music-room-transcendence/
├── .github/
│   └── workflows/
│       └── ci.yml                 # cargo check / clippy / test + npm run build
├── infra/
│   ├── nginx/
│   │   ├── nginx.conf             # HTTPS + reverse proxy + WebSockets + SSE
│   │   └── modsecurity/           # Reglas WAF endurecidas (ajustadas a WS/SSE)
│   ├── vault/
│   │   ├── config.hcl             # Config de HashiCorp Vault
│   │   └── init-policies.sh       # Políticas y carga inicial de secretos
│   └── postgres/
│       └── init.sql               # Inicialización opcional / extensiones
├── backend/
│   ├── migration/                 # Crate de migraciones SeaORM (sea-orm-migration)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── m20260101_000001_init.rs
│   ├── src/
│   │   ├── main.rs                # Arranque Tokio, Vault/config, pool DB/Redis, router, server
│   │   ├── config.rs              # Config tipada (entorno + Vault)
│   │   ├── state.rs               # AppState (SeaORM DatabaseConnection, Redis, WS hub, HTTP)
│   │   ├── error.rs               # Error centralizado de app -> IntoResponse (HTTP + JSON)
│   │   ├── entities/              # Entidades generadas/definidas con SeaORM
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── friendship.rs
│   │   │   ├── user_device.rs
│   │   │   ├── music_room.rs
│   │   │   ├── room_track.rs
│   │   │   ├── track_vote.rs
│   │   │   ├── playlist.rs
│   │   │   ├── playlist_item.rs
│   │   │   ├── control_delegation.rs
│   │   │   ├── chat_message.rs
│   │   │   └── audit_log.rs
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs            # Validación JWT / sesión
│   │   │   ├── subscription.rs    # Comprobación plan FREE / PAID
│   │   │   └── client_log.rs      # Registro de plataforma, dispositivo y versión app
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs            # Registro, login (Argon2), JWT, OAuth2 (Google / 42)
│   │   │   ├── users.rs           # Perfil, visibilidad (público/amigos/privado), amigos, plan
│   │   │   ├── rooms.rs           # Salas, votos, playlist colaborativa, delegación control
│   │   │   ├── search.rs          # Búsqueda avanzada (filtros, orden, paginación) + catálogo
│   │   │   ├── chat.rs            # Mensajes de sala/directos + moderación
│   │   │   └── ai.rs              # Recomendaciones ML + streaming SSE del LLM
│   │   ├── handlers/              # Extracción HTTP (Path, Query, Json, State) + llamadas a services
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── users.rs
│   │   │   ├── rooms.rs
│   │   │   ├── search.rs
│   │   │   ├── chat.rs
│   │   │   └── ai.rs
│   │   ├── services/              # Reglas de negocio, licencias/permisos y concurrencia
│   │   │   ├── mod.rs
│   │   │   ├── auth_service.rs
│   │   │   ├── room_service.rs    # Visibilidad, licencias de voto, delegación por dispositivo
│   │   │   ├── playlist_service.rs# Concurrencia al reordenar/editar pistas
│   │   │   ├── recommender.rs     # Recomendación ML (preferencias, likes, historial)
│   │   │   ├── moderation.rs      # Moderación IA de textos (chat / nombres de sala)
│   │   │   ├── llm.rs             # Cliente LLM con streaming
│   │   │   └── spotify.rs         # Cliente externo de música (metadatos / búsqueda)
│   │   └── ws/
│   │       ├── mod.rs
│   │       ├── hub.rs             # Gestión concurrente de salas y suscriptores + Redis Pub/Sub
│   │       └── handler.rs         # Upgrade WS, envío/recepción de eventos en tiempo real
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── Dockerfile                 # Multi-stage build limpio para producción
├── frontend/
│   ├── public/
│   ├── src/
│   │   ├── api/                   # Cliente HTTP configurable (env) + cliente WS/SSE
│   │   ├── components/            # Componentes UI reutilizables (reproductor, votos, modales)
│   │   ├── context/               # AuthContext, SocketContext, PlayerContext
│   │   ├── pages/
│   │   │   ├── auth/              # Login, registro, callback OAuth
│   │   │   ├── rooms/             # Sala de votación en vivo + editor colaborativo de playlist
│   │   │   ├── social/            # Amigos, perfiles, chat
│   │   │   ├── ai/                # Recomendaciones + asistente LLM en streaming
│   │   │   └── legal/             # Privacy Policy y Terms of Service reales
│   │   ├── types/                 # Tipos TS alineados con las respuestas/eventos del backend
│   │   └── App.tsx
│   ├── package.json
│   └── Dockerfile
├── tests/                         # Pruebas de integración / carga / concurrencia (apoyo QA)
├── docker-compose.yml             # Levanta todo el stack con un solo comando
├── Makefile                       # Automatización de dependencias, build, run, lint, test
├── .env.example                   # Plantilla de variables sin secretos reales
└── README.md                      # Documentación completa en inglés en la raíz
```
---

### 3. Estrategia de Branches (Git Flow Simplificado)

* `main`: Código estable y listo para evaluación. Protegida (solo entra código probado desde `develop`).
* `develop`: Rama principal de integración donde convergen backend y frontend.
* **Ramas de Roger (Backend Rust & Infra):**
* `feat/be-infra-security`: Docker Compose (build multi-stage de Rust), Nginx + HTTPS, WAF/ModSecurity y HashiCorp Vault (`vaultrs`).
* `feat/be-auth-users`: Axum base, SeaORM + migraciones, Argon2 + JWT, OAuth 2.0, perfiles, amigos y suscripciones Free/Paid.
* `feat/be-music-realtime`: Salas, integración Spotify (solo lectura), WebSockets (`hub.rs`), concurrencia de votos en PostgreSQL y delegación de control.
* `feat/be-search-chat`: Búsqueda avanzada (filtros/ordenamiento/paginación) y chat persistente en tiempo real.
* `feat/be-ai-modules`: Sistema de recomendación ML, interfaz LLM con streaming SSE/rate-limit y moderación IA.


* **Ramas de Alejandro (Frontend React):**
* `feat/fe-setup-legal`: Estructura React + TS, routing, layout responsive, Privacy Policy y Terms of Service.
* `feat/fe-auth-profile`: Pantallas de login/registro, flujo OAuth 2.0, edición de perfil, amigos y selector Free/Paid.
* `feat/fe-rooms-realtime`: Interfaz de votación en vivo, editor colaborativo de playlists y delegación de control por WebSockets.
* `feat/fe-social-search`: Chat en tiempo real y buscador avanzado con filtros y paginación.
* `feat/fe-ai-ui`: Vista de recomendaciones musicales y chat/interfaz streaming del LLM.



---

### 4. Hoja de Ruta por Fases (División Roger / Alejandro)

#### Fase 1: Cimientos, Contenedores y Seguridad Base

* **Roger (`feat/be-infra-security` + `feat/be-auth-users`):**
* Configurar `docker-compose.yml` con PostgreSQL, Redis, el binario Rust (Axum), Nginx (HTTPS autofirmado) y HashiCorp Vault.


* Definir entidades y migraciones iniciales con **SeaORM**, hashing de contraseñas con `argon2`, tokens JWT (`jsonwebtoken`), OAuth 2.0, perfiles (público/amigos/privado), amigos y plan (`FREE` / `PAID`).


* Generar especificaciones OpenAPI (con `utoipa`) o tipos TS (`ts-rs`) para que Alejandro tenga los contratos JSON exactos desde el inicio.


* **Alejandro (`feat/fe-setup-legal` + `feat/fe-auth-profile`):**
* Inicializar proyecto React + TypeScript responsive con framework CSS (Tailwind/MUI) y sistema de rutas.


* Crear las páginas reales de **Privacy Policy** y **Terms of Service** accesibles en el footer (requisito eliminatorio de *ft_transcendence*).


* Implementar pantallas de Login, Registro, callback OAuth 2.0, perfil de usuario (con subida de avatar) y gestión de amigos.





#### Fase 2: Núcleo *Music Room* y Tiempo Real (WebSockets en Rust)

* **Roger (`feat/be-music-realtime` + `feat/be-search-chat`):**
* Implementar rutas Axum y el `ws/hub.rs` concurrente para:
1. **Music Track Vote:** Salas públicas/privadas, licencias de voto y transacciones atómicas en SeaORM/PostgreSQL para evitar *race conditions* en votos simultáneos.


2. **Music Playlist Editor:** Edición colaborativa en tiempo real con sincronización de orden de pistas vía WebSockets.


3. **Music Control Delegation:** Permisos por sesión/dispositivo para ceder el control de reproducción a amigos.


4. **Free vs Paid:** Extractor/middleware en Axum que restrinja funciones premium (p. ej. salas privadas ilimitadas o edición avanzada) a usuarios `PAID`.




* Conectar cliente Spotify Web API (solo lectura de catálogo/previews, guardando toda interacción en tu PostgreSQL), chat en tiempo real y búsqueda avanzada con filtros y paginación.




* **Alejandro (`feat/fe-rooms-realtime` + `feat/fe-social-search`):**
* Conectar cliente WebSocket global con reconexión automática.


* Construir la sala interactiva: cola de canciones con votos en vivo, reordenamiento colaborativo de playlist y panel para delegar/revocar control a usuarios conectados.


* Integrar barra de búsqueda avanzada (filtros + paginación), chat de sala/amigos y selector de plan Free/Paid.





#### Fase 3: Módulos de IA y Endurecimiento WAF/Vault

* **Roger (`feat/be-ai-modules` + cierre `feat/be-infra-security`):**
* **Recomendación ML:** Endpoint que cruza gustos musicales del perfil, likes e historial en PostgreSQL mediante cálculo de similitud/scoring para devolver canciones y salas recomendadas.


* **LLM Interface:** Endpoint con respuesta en *streaming* (`axum::response::sse::Sse`) para generar playlists o sugerencias por prompt, con control de errores y *rate limiting*.


* **Moderación IA:** Verificación automática antes de persistir/emitir mensajes de chat que bloquea o advierte contenido tóxico.


* Endurecer reglas de **ModSecurity/WAF** en Nginx (permitiendo explícitamente el upgrade `WebSocket` y `text/event-stream` de SSE) y cargar todos los secretos desde **HashiCorp Vault** en `main.rs`.




* **Alejandro (`feat/fe-ai-ui`):**
* Crear sección "Para ti" con el feed del sistema de recomendación ML.


* Construir la interfaz del asistente LLM renderizando el texto en *streaming* token a token y manejando estados de carga/límite de peticiones.


* Mostrar avisos visuales cuando la IA de moderación bloquee un mensaje en el chat.





#### Fase 4: Integración Final y Preparación para Evaluación

* Revisión conjunta en `develop` -> `main` verificando **cero errores o warnings en consola de Chrome**, despliegue limpio con un solo comando (`docker-compose up --build`) y prueba de concurrencia con múltiples usuarios simultáneos.


* **Ayub y Persona Cuarta** apoyan aquí en pruebas cruzadas, redacción del `README.md` (con roles, arquitectura, esquema de BD y justificación de módulos) y sus respectivos commits para cumplir la distribución de equipo.