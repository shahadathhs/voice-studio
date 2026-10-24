"use client";

import { useEffect } from "react";

const TARGETS = ".reveal, .reveal-item, .hero-shot";

/** For browsers without CSS scroll-driven animation: reveals the same blocks as they scroll into view.
 *  The head script only marks such browsers, so everywhere else this does nothing. */
export function ScrollReveal() {
  useEffect(() => {
    if (!document.documentElement.classList.contains("reveal-js")) return;
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add("is-in");
          observer.unobserve(entry.target);
        }
      },
      { threshold: 0.12 },
    );
    document.querySelectorAll(TARGETS).forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  }, []);
  return null;
}
