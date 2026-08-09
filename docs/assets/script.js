/* AfterBudget — site officiel.
   JavaScript minimal : menu Linux dépliant, visionneuse de captures,
   mise en avant du système de l'utilisateur. */

(function () {
  "use strict";

  /* ── Menu Linux (Ubuntu / Fedora) ─────────────────────────────────── */
  var boutonLinux = document.getElementById("menu_linux");
  var bouton = boutonLinux.querySelector(".bouton_menu_linux");

  function fermerMenu() {
    boutonLinux.classList.remove("ouvert");
    bouton.setAttribute("aria-expanded", "false");
  }

  bouton.addEventListener("click", function (event) {
    event.stopPropagation();
    var ouvert = boutonLinux.classList.toggle("ouvert");
    bouton.setAttribute("aria-expanded", ouvert ? "true" : "false");
  });

  document.addEventListener("click", fermerMenu);
  boutonLinux.addEventListener("click", function (event) {
    event.stopPropagation();
  });

  document.addEventListener("keydown", function (event) {
    if (event.key === "Escape") fermerMenu();
  });

  /* ── Visionneuse (zoom au clic sur les captures) ──────────────────── */
  var visionneuse = document.getElementById("visionneuse");
  var imageVisionneuse = visionneuse.querySelector("img");
  var boutonFermer = visionneuse.querySelector(".visionneuse_fermer");

  function ouvrirVisionneuse(source, alt) {
    imageVisionneuse.src = source;
    imageVisionneuse.alt = alt || "";
    visionneuse.hidden = false;
    document.body.style.overflow = "hidden";
  }

  function fermerVisionneuse() {
    visionneuse.hidden = true;
    imageVisionneuse.src = "";
    document.body.style.overflow = "";
  }

  document.querySelectorAll(".cadre_capture img").forEach(function (image) {
    image.addEventListener("click", function () {
      ouvrirVisionneuse(image.currentSrc || image.src, image.alt);
    });
  });

  boutonFermer.addEventListener("click", fermerVisionneuse);
  visionneuse.addEventListener("click", function (event) {
    if (event.target === visionneuse) fermerVisionneuse();
  });
  document.addEventListener("keydown", function (event) {
    if (event.key === "Escape" && !visionneuse.hidden) fermerVisionneuse();
  });

})();
