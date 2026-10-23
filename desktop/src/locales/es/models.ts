import type { models as english } from "../en/models";

export const models: Record<keyof typeof english, string> = {
  "models.title": "Modelos",
  "models.subtitle":
    "Se descargan una vez y luego funcionan sin conexión. Elimina los que no necesites.",
  "models.installedCount": "{installed} de {total} instalados",
  "models.onDisk": "{size} en disco",
  "models.none.title": "No hay modelos que mostrar",
  "models.none.text": "Los modelos solo están disponibles en la aplicación de escritorio.",
  "models.storedIn": "Guardados en {path}",
  "models.install": "Instalar",
  "models.installed": "Instalado",
  "models.remove": "Eliminar",
  "models.downloading": "Descargando {name}",
  "models.downloaded": "{done} de {total}",
  "models.recommended": "Recomendado",
  "models.licence": "licencia {license}",
  "models.canTranslate": "puede traducir al inglés",
  "models.group.stt.title": "Transcripción",
  "models.group.stt.purpose":
    "Convierten audio y vídeo en texto. Necesitas uno; los modelos más grandes son más precisos pero más lentos. ¿Dudas? Empieza con Whisper small.",
  "models.group.diarization.title": "Etiquetas de hablante",
  "models.group.diarization.purpose":
    "Opcional. Añade quién dijo qué a una transcripción. Funciona junto a un modelo de transcripción y se activa con Identificar hablantes.",
  "models.group.tts.title": "Voz",
  "models.group.tts.purpose":
    "Lee texto en voz alta con voces naturales. La pantalla Hablar necesita uno de estos.",
  "models.goTranscribe": "Ir a Transcribir",
  "models.goSpeak": "Ir a Hablar",
  "modelInfo.whisper-tiny":
    "El más rápido y el menos preciso. Para borradores rápidos de voz clara y cercana.",
  "modelInfo.whisper-base":
    "Rápido y ligero. Bueno para grabaciones limpias en una sala tranquila.",
  "modelInfo.whisper-small":
    "El mejor equilibrio para la mayoría: maneja acentos y algo de ruido de fondo.",
  "modelInfo.whisper-medium": "Alta precisión con audio difícil. Notablemente más lento.",
  "modelInfo.whisper-large-v3-turbo":
    "Casi la mejor precisión a mucha más velocidad. No puede traducir.",
  "modelInfo.whisper-large-v3-turbo-q5":
    "El modelo turbo comprimido a un tercio de la descarga, algo menos preciso.",
  "modelInfo.whisper-large-v3":
    "La mayor precisión y el más lento. Para grabaciones difíciles donde importa la calidad.",
  "modelInfo.wespeaker-resnet34":
    "Distingue las voces para que una transcripción indique quién habla. Pequeño y rápido en la CPU.",
  "modelInfo.kokoro-82m":
    "Lee texto en inglés en voz alta con seis voces naturales estadounidenses. Funciona en la CPU.",
  "onboarding.title": "Elige qué descargar",
  "onboarding.lead":
    "Los modelos se ejecutan en este ordenador, así que hay que descargarlos una vez. Puedes añadir o quitar cualquiera después en Modelos.",
  "onboarding.total": "{size} por descargar",
  "onboarding.downloading":
    "Descargando… esta pantalla se cierra cuando la transcripción esté lista.",
  "onboarding.download": "Descargar",
  "onboarding.skip": "Omitir por ahora",
  "onboarding.privacy":
    "Las descargas vienen del alojamiento público de los autores de los modelos. Sin cuenta, y no se envía nada sobre ti.",
};
