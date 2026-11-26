export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export interface ApiEndpoint {
  id: string;
  module:
    | "tracks"
    | "rooms"
    | "users"
    | "artists"
    | "admins"
    | "studio"
    | "uploads"
    | "auth"
    | "webrtc";
  method: HttpMethod;
  path: string;
  summary: string;
  description: string;
  authRequired: boolean;
  sampleBody?: Record<string, unknown>;
  sampleResponse: Record<string, unknown>;
}

export const API_ENDPOINTS: ApiEndpoint[] = [
  // TRACKS & MEDIA DELIVERY
  {
    id: "tracks-list",
    module: "tracks",
    method: "GET",
    path: "/api/tracks",
    summary: "Listar catálogo de pistas",
    description: "Devuelve pistas disponibles con metadatos, artista, duración y estado de audio.",
    authRequired: false,
    sampleResponse: {
      items: [{ id: 1, title: "Midnight Pulse", artist_id: 7, duration_ms: 214000 }],
      total: 1,
    },
  },
  {
    id: "tracks-detail",
    module: "tracks",
    method: "GET",
    path: "/api/tracks/{id}",
    summary: "Obtener detalle de una pista",
    description: "Consulta ficha técnica, hash SHA-256, bitrate y enlaces de entrega.",
    authRequired: false,
    sampleResponse: {
      id: 4,
      title: "Neon Drift",
      artist_id: 7,
      bitrate_kbps: 320,
      sha256: "9f86d081884c7d659a2feaa0c55ad015...",
    },
  },
  {
    id: "tracks-create",
    module: "tracks",
    method: "POST",
    path: "/api/tracks",
    summary: "Crear registro de pista",
    description: "Registra una nueva pista asociada a un artista o lanzamiento.",
    authRequired: true,
    sampleBody: { title: "Analog Echo", artist_id: 7, genre: "synthwave" },
    sampleResponse: { id: 12, status: "created" },
  },
  {
    id: "tracks-stream",
    module: "tracks",
    method: "GET",
    path: "/api/tracks/{id}/stream",
    summary: "Streaming de audio con Range",
    description: "Entrega audio por rangos HTTP (`206 Partial Content`) para reproducción fluida.",
    authRequired: false,
    sampleResponse: { status: 206, content_type: "audio/mpeg", accept_ranges: "bytes" },
  },

  // ROOMS & QUEUE
  {
    id: "rooms-list",
    module: "rooms",
    method: "GET",
    path: "/api/rooms",
    summary: "Listar salas de música activas",
    description: "Obtiene salas públicas/privadas, oyentes activos y pista actual.",
    authRequired: false,
    sampleResponse: {
      rooms: [{ id: 1, name: "Late Night Dub", listeners: 14, now_playing_track_id: 4 }],
    },
  },
  {
    id: "rooms-create",
    module: "rooms",
    method: "POST",
    path: "/api/rooms",
    summary: "Crear una sala de música",
    description: "Inicializa el estado de la sala y su `RoomHub` para eventos en tiempo real.",
    authRequired: true,
    sampleBody: { name: "Lo-Fi & Chill", is_private: false },
    sampleResponse: { id: 2, name: "Lo-Fi & Chill", status: "active" },
  },
  {
    id: "rooms-enqueue",
    module: "rooms",
    method: "POST",
    path: "/api/rooms/{room_id}/queue",
    summary: "Encolar pista en la sala",
    description: "Añade un `track_id` a la cola compartida y notifica por WebSocket (`track_queued`).",
    authRequired: true,
    sampleBody: { track_id: 4 },
    sampleResponse: { queued: true, track_id: 4, votes: 0 },
  },
  {
    id: "rooms-vote",
    module: "rooms",
    method: "POST",
    path: "/api/rooms/{room_id}/queue/{track_id}/vote",
    summary: "Votar o quitar voto en la cola",
    description: "Alterna el voto del usuario y reordena la cola por puntuación (`vote_updated`).",
    authRequired: true,
    sampleBody: { user_id: 1 },
    sampleResponse: { track_id: 4, votes: 5, voted: true },
  },
  {
    id: "rooms-next",
    module: "rooms",
    method: "POST",
    path: "/api/rooms/{room_id}/next",
    summary: "Avanzar a la pista más votada",
    description: "Promueve el tema con más votos a `now_playing` y difunde `now_playing_changed`.",
    authRequired: true,
    sampleResponse: { now_playing: { track_id: 4, is_playing: true } },
  },

  // USERS
  {
    id: "users-me",
    module: "users",
    method: "GET",
    path: "/api/users/me",
    summary: "Perfil del usuario autenticado",
    description: "Devuelve datos de cuenta, plan actual, rol y preferencias del usuario.",
    authRequired: true,
    sampleResponse: { id: 1, username: "dj_nova", plan: "FREE", role: "user" },
  },
  {
    id: "users-by-id",
    module: "users",
    method: "GET",
    path: "/api/users/{id}",
    summary: "Consultar perfil público de usuario",
    description: "Obtiene información pública, avatar y actividad en salas o playlists.",
    authRequired: false,
    sampleResponse: { id: 1, username: "dj_nova", bio: "Curador de breaks y ambient" },
  },
  {
    id: "users-update",
    module: "users",
    method: "PATCH",
    path: "/api/users/{id}",
    summary: "Actualizar perfil de usuario",
    description: "Modifica nombre visible, bio o ajustes de reproducción del usuario.",
    authRequired: true,
    sampleBody: { bio: "Nuevos sets cada viernes" },
    sampleResponse: { id: 1, updated: true },
  },

  // ARTISTS & STUDIO
  {
    id: "artists-list",
    module: "artists",
    method: "GET",
    path: "/api/artists",
    summary: "Listar artistas",
    description: "Catálogo de artistas verificados y creadores con sus métricas básicas.",
    authRequired: false,
    sampleResponse: {
      artists: [{ id: 7, name: "Kairo Wave", verified: true, followers: 1280 }],
    },
  },
  {
    id: "artists-detail",
    module: "artists",
    method: "GET",
    path: "/api/artists/{id}",
    summary: "Detalle de artista y discografía",
    description: "Devuelve biografía, lanzamientos y pistas asociadas al artista.",
    authRequired: false,
    sampleResponse: { id: 7, name: "Kairo Wave", tracks_count: 14 },
  },
  {
    id: "studio-releases",
    module: "studio",
    method: "GET",
    path: "/api/studio/releases",
    summary: "Lanzamientos del creador en Studio",
    description: "Lista borradores, archivos subidos y publicaciones del estudio del creador.",
    authRequired: true,
    sampleResponse: {
      releases: [{ id: 21, title: "Night Bus EP", state: "published" }],
    },
  },
  {
    id: "uploads-init",
    module: "uploads",
    method: "POST",
    path: "/api/uploads",
    summary: "Subir archivo de audio con SHA-256",
    description: "Recibe archivo o chunks, verifica integridad SHA-256 y prepara la pista.",
    authRequired: true,
    sampleBody: { filename: "track.flac", sha256: "e3b0c44298fc1c149afbf4c8996fb924..." },
    sampleResponse: { upload_id: "upl_9821", verified: true },
  },

  // ADMINS & AUTH
  {
    id: "auth-login-2fa",
    module: "auth",
    method: "POST",
    path: "/api/auth/login",
    summary: "Inicio de sesión con soporte 2FA",
    description: "Valida credenciales y emite token o solicita código de segundo factor.",
    authRequired: false,
    sampleBody: { email: "user@example.com", password: "********" },
    sampleResponse: { requires_2fa: true, challenge_token: "ch_55a1" },
  },
  {
    id: "admin-users",
    module: "admins",
    method: "GET",
    path: "/api/admin/users",
    summary: "Gestión administrativa de usuarios",
    description: "Listado de cuentas, estados, roles y planes para moderación y soporte.",
    authRequired: true,
    sampleResponse: {
      users: [{ id: 1, email: "user@example.com", role: "user", status: "active" }],
    },
  },
  {
    id: "admin-moderation",
    module: "admins",
    method: "POST",
    path: "/api/admin/tracks/{id}/moderate",
    summary: "Moderar pista o contenido",
    description: "Aprueba, oculta o marca una pista por políticas de plataforma.",
    authRequired: true,
    sampleBody: { action: "approve", reason: "Verificado" },
    sampleResponse: { track_id: 4, moderation_status: "approved" },
  },

  // WEBRTC SIGNALING
  {
    id: "webrtc-room-signal",
    module: "webrtc",
    method: "POST",
    path: "/api/rooms/{room_id}/webrtc/signal",
    summary: "Señalización WebRTC en sala (`room`)",
    description: "Enruta `offer`, `answer`, `ice-candidate` o `peer-announce` dentro de una sala.",
    authRequired: true,
    sampleBody: {
      type: "peer-announce",
      scope: "room",
      room_id: 1,
      resource_id: null,
      peer_id: "peer-1-abc",
    },
    sampleResponse: { delivered: true, scope: "room" },
  },
  {
    id: "webrtc-global-signal",
    module: "webrtc",
    method: "POST",
    path: "/api/webrtc/signal",
    summary: "Señalización WebRTC `swarm` y `direct`",
    description: "Coordina pares para intercambio directo o por enjambre (`SignalHub`).",
    authRequired: true,
    sampleBody: {
      type: "peer-announce",
      scope: "swarm",
      room_id: null,
      resource_id: "track-4",
      peer_id: "peer-1-abc",
      available_resources: ["track-4"],
    },
    sampleResponse: { delivered: true, scope: "swarm" },
  },
];