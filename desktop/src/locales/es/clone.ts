import type { clone as english } from "../en/clone";

/** Clonación de voz: una voz creada a partir de una grabación de alguien que ha dado su permiso. */
export const clone: Record<keyof typeof english, string> = {
  "clone.title": "Clonar una voz",
  "clone.hint":
    "Crea una voz con 3 a 30 segundos de habla clara de una sola persona. Funciona en este equipo. Clona solo una voz que sea tuya o que tengas permiso para usar.",
  "clone.needModel": "Clonar necesita el modelo Chatterbox (unos 1,5 GB).",
  "clone.openModels": "Obtener el modelo",
  "clone.file": "Grabación",
  "clone.chooseFile": "Elegir una grabación",
  "clone.noFile": "Ninguna grabación elegida",
  "clone.name": "Nombre",
  "clone.namePlaceholder": "Ana",
  "clone.consent": "Soy el dueño de esta voz o tengo el permiso del hablante",
  "clone.make": "Clonar voz",
  "clone.working":
    "Clonando la voz. La primera vez se carga un modelo grande y puede tardar un minuto.",
  "clone.disclosure":
    "El audio hecho con una voz clonada se guarda con una nota que indica que es sintético. No lleva marca de agua.",
  "clone.needAll": "Elige una grabación, ponle nombre a la voz y confirma el permiso.",
  "clone.group": "Voces clonadas",
  "clone.badge": "clonada",
  "clone.delete": "Eliminar la voz clonada {name}",
  "clone.language": "Idioma del texto",
  "clone.slow": "Las voces clonadas son más lentas que las incluidas.",
};
