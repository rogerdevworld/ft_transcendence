### 1. Mapa de Módulos Seleccionados (19 puntos)

* **Web (9 pts):**
* **Major (2 pts):** Frameworks en Frontend (React) y Backend (Django + Django Ninja).


* **Major (2 pts):** Tiempo real con WebSockets (Django Channels + Redis) para votos en vivo, playlists y delegación de control.


* **Major (2 pts):** Interacción entre usuarios (chat en salas, perfiles y sistema de amigos).


* **Minor (1 pt):** ORM para base de datos (Django ORM + PostgreSQL).


* **Minor (1 pt):** Funciones colaborativas en tiempo real (edición multiusuario simultánea de playlists).


* **Minor (1 pt):** Búsqueda avanzada (filtros por género/visibilidad, ordenamiento por votos y paginación).




* **User Management (3 pts):**
* **Major (2 pts):** Gestión estándar de usuarios y autenticación (registro, perfiles público/amigos/privado, avatar con default y estado online).


* **Minor (1 pt):** Autenticación remota con OAuth 2.0 (Google / 42).




* **Artificial Intelligence (5 pts):**
* **Major (2 pts):** Sistema de recomendación ML (recomendación de canciones/playlists según preferencias e historial).


* **Major (2 pts):** Interfaz completa de LLM (asistente musical/generador de playlists con streaming y rate limiting).


* **Minor (1 pt):** IA de moderación de contenido (auto-moderación en el chat y nombres de salas).




* **Cybersecurity (2 pts):**
* **Major (2 pts):** WAF/ModSecurity (hardened) en el reverse proxy + HashiCorp Vault para gestión cifrada de secretos.



---

### 2. Estructura de Carpetas del Monorepo

```text
music-room-transcendence/
├── .github/                    # Workflows CI básicos (lint/tests)
├── infra/                      # Infraestructura, Proxy y Seguridad (Roger)
│   ├── nginx/                  # Configuración HTTPS + ModSecurity WAF (hardened)
│   ├── vault/                  # Políticas, scripts de inicialización y config HashiCorp Vault
│   └── postgres/               # Scripts de init de BD
├── backend/                    # Django + Django Ninja + Channels (Roger)
│   ├── config/                 # settings, asgi.py (WebSockets), urls.py, ninja_api.py
│   ├── apps/
│   │   ├── users/              # Auth JWT, OAuth 2.0, perfiles, amigos, Free/Paid tiers
│   │   ├── rooms/              # Track Vote, Playlist Editor, Control Delegation, licencias
│   │   ├── chat/               # Mensajería en salas/amigos + filtro de moderación IA
│   │   ├── ai_engine/          # Motor recomendación ML, endpoint streaming LLM, moderación
│   │   └── common/             # Permisos, paginación, búsqueda avanzada, middleware logs
│   ├── manage.py
│   ├── requirements.txt
│   └── Dockerfile
├── frontend/                   # React + TypeScript + Tailwind/CSS (Alejandro)
│   ├── public/
│   ├── src/
│   │   ├── api/                # Clientes HTTP (Fetch/Axios) y hooks de WebSockets
│   │   ├── components/         # UI reutilizable (reproductor, botones voto, modales, navbar)
│   │   ├── context/            # AuthContext, SocketContext, PlayerContext
│   │   ├── pages/
│   │   │   ├── auth/           # Login, Registro, Callback OAuth
│   │   │   ├── rooms/          # Sala de votación en vivo, Editor playlist colaborativo
│   │   │   ├── social/         # Chat, Perfil, Lista de amigos
│   │   │   ├── ai/             # Panel de recomendaciones y asistente LLM streaming
│   │   │   └── legal/          # Privacy Policy y Terms of Service (obligatorios 42)
│   │   └── types/              # Contratos TypeScript sincronizados con esquemas Django Ninja
│   ├── package.json
│   └── Dockerfile
├── docker-compose.yml          # Levanta todo con un solo comando (obligatorio 42)
├── Makefile                    # Atajos: make up, make down, make logs, make vault-init
├── .env.example                # Plantilla de variables sin secretos
└── README.md                   # Documentación completa en inglés (obligatoria 42)

```

---

### 3. Estrategia de Branches (Git Flow Simplificado)

* `main`: Código estable y listo para evaluación. Protegida (solo entra código probado desde `develop`).
* `develop`: Rama principal de integración donde convergen backend y frontend.
* **Ramas de Roger (Backend & Infra):**
* `feat/be-infra-security`: Docker Compose, Nginx + HTTPS, WAF/ModSecurity y HashiCorp Vault.
* `feat/be-auth-users`: Django + Ninja base, JWT, OAuth 2.0, perfiles, amigos y suscripciones Free/Paid.
* `feat/be-music-realtime`: Modelos de salas/playlists, WebSockets (Channels), concurrencia de votos y delegación de control.
* `feat/be-search-chat`: Búsqueda avanzada (filtros/paginación) y chat persistente.
* `feat/be-ai-modules`: Sistema de recomendación ML, interfaz LLM con streaming/rate-limit y moderación IA.


* **Ramas de Alejandro (Frontend):**
* `feat/fe-setup-legal`: Estructura React, routing, layout responsive, Privacy Policy y Terms of Service.
* `feat/fe-auth-profile`: Pantallas de login/registro, flujo OAuth 2.0, edición de perfil, amigos y selector Free/Paid.
* `feat/fe-rooms-realtime`: Interfaz de votación en vivo, editor colaborativo de playlists y delegación de control por WebSockets.
* `feat/fe-social-search`: Chat en tiempo real y buscador avanzado con filtros y paginación.
* `feat/fe-ai-ui`: Vista de recomendaciones musicales y chat/interfaz streaming del LLM.



---


### 4. Hoja de Ruta por Fases (División Roger / Alejandro)

#### Fase 1: Cimientos, Contenedores y Seguridad Base

* **Roger (`feat/be-infra-security` + `feat/be-auth-users`):**
* Configurar `docker-compose.yml` con PostgreSQL, Redis, Django (ASGI/Uvicorn), Nginx (HTTPS autofirmado) y HashiCorp Vault.


* Configurar Django Ninja con autenticación por tokens (JWT), endpoints de registro/login, OAuth 2.0, perfiles (público/amigos/privado), amigos y campo de suscripción (`FREE` / `PAID`).


* Dejar autogenerada la documentación Swagger/OpenAPI de Django Ninja (`/api/docs`) para que Alejandro vea los esquemas desde el día 1.




* **Alejandro (`feat/fe-setup-legal` + `feat/fe-auth-profile`):**
* Inicializar proyecto React responsive con framework CSS (Tailwind/MUI) y sistema de rutas.


* Crear las páginas reales de **Privacy Policy** y **Terms of Service** accesibles en el footer (requisito eliminatorio de *ft_transcendence*).


* Implementar pantallas de Login, Registro, callback OAuth 2.0, perfil de usuario (con subida de avatar) y gestión de amigos.





#### Fase 2: Núcleo *Music Room* y Tiempo Real (WebSockets)

* **Roger (`feat/be-music-realtime` + `feat/be-search-chat`):**
* Implementar endpoints Ninja y consumidores WebSocket (Django Channels) para:
1. **Music Track Vote:** Salas públicas/privadas, licencias de voto y bloqueo/transacciones atómicas en BD para evitar *race conditions* cuando varios votan a la vez.


2. **Music Playlist Editor:** Edición colaborativa en tiempo real con sincronización de orden de pistas.


3. **Music Control Delegation:** Permisos por sesión/dispositivo para ceder el control de reproducción a amigos.


4. **Free vs Paid:** Middleware/decorador en Ninja que restrinja funciones premium (ej. crear salas privadas ilimitadas o edición avanzada) a usuarios `PAID`.




* Implementar chat en tiempo real y endpoints de búsqueda avanzada (filtros, ordenamiento y paginación).




* **Alejandro (`feat/fe-rooms-realtime` + `feat/fe-social-search`):**
* Conectar cliente WebSocket global con reconexión automática.


* Construir la sala interactiva: cola de canciones con votos en vivo, reordenamiento colaborativo de playlist y panel para delegar/revocar control a usuarios conectados.


* Integrar barra de búsqueda avanzada (filtros + paginación), chat de sala/amigos y vista de cambio de plan Free/Paid.





#### Fase 3: Módulos de IA y Endurecimiento WAF/Vault

* **Roger (`feat/be-ai-modules` + cierre `feat/be-infra-security`):**
* **Recomendación ML:** Endpoint que cruza gustos musicales del perfil, votos e historial para devolver canciones/salas recomendadas.


* **LLM Interface:** Endpoint con respuesta en streaming (`StreamingHttpResponse` / SSE) para generar playlists o sugerencias por prompt, con control de errores y *rate limiting*.


* **Moderación IA:** Hook en el envío de mensajes de chat que analiza y bloquea/advierte contenido tóxico automáticamente.


* Endurecer reglas de **ModSecurity/WAF** en Nginx y migrar todas las API keys/credenciales a **HashiCorp Vault** (verificando que no haya falsos positivos que bloqueen WebSockets o streaming).




* **Alejandro (`feat/fe-ai-ui`):**
* Crear sección "Para ti" con el feed del sistema de recomendación ML.


* Construir la interfaz del asistente LLM renderizando el texto en streaming token a token y manejando estados de carga/límite de peticiones.


* Mostrar avisos visuales cuando la IA de moderación bloquee un mensaje en el chat.





#### Fase 4: Integración Final y Preparación para Evaluación

* Revisión conjunta en `develop` -> `main` verificando **cero errores o warnings en consola de Chrome**, despliegue limpio con un solo comando (`docker-compose up --build`) y prueba de concurrencia con múltiples usuarios simultáneos.


* **Ayub y Persona Cuarta** apoyan aquí en pruebas cruzadas, redacción del `README.md` (con roles, arquitectura, esquema de BD y justificación de módulos) y sus respectivos commits para cumplir la distribución de equipo.

