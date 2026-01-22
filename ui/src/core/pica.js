import Pica from "pica";

/**
 * Singleton Pica instance to prevent "Thread Storms".
 * Initializing this once allows the library to reuse a single set of Web Workers
 * for all image resizing tasks across the application.
 */
export const pica = new Pica();
