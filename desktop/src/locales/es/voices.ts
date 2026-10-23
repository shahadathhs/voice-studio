import type { voices as english } from "../en/voices";

export const voices: Record<keyof typeof english, string> = {
  "voices.create.title": "Crear una voz",
  "voices.create.hint":
    "Mezcla de 2 a 4 voces de un mismo idioma en una voz nueva. Es una mezcla de las voces sintéticas incluidas, no la copia de una persona.",
  "voices.name": "Nombre",
  "voices.namePlaceholder": "Mi voz",
  "voices.language": "Idioma",
  "voices.mix": "Voces para mezclar",
  "voices.weight": "Peso de {voice}",
  "voices.needTwo": "Elige al menos 2 voces y ponle un nombre a la voz.",
  "voices.maxFour": "Una mezcla puede combinar hasta 4 voces.",
  "voices.preview": "Escuchar",
  "voices.save": "Guardar voz",
  "voices.custom": "personalizada",
  "voices.delete": "Eliminar {name}",
} as const;
