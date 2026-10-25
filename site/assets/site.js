// Sol JK site enhancements. The page works without this script: the download
// button points at the latest release page and the changelog links to GitHub.
"use strict";

const REPO = "Sol-Vulpes/SJK";
const RELEASES_API = `https://api.github.com/repos/${REPO}/releases?per_page=6`;
const RELEASES_PAGE = `https://github.com/${REPO}/releases`;

// Donations (docs/sjk.md, "Donations"). The card reads the live goal from the SJK hub
// (`hub`, which Ko-fi's webhook keeps up to date); the other values are what it shows
// when the hub cannot be reached: `url` the donation page (an https address; anything
// else says the link is coming), `goal` the monthly goal and `raised` what came in
// this month, both whole euros.
const DONATION = {
  // Donations are shelved (14/10/2026): the section stays hidden and the hub is not
  // asked. Set to false along with removing index.html's hidden attributes.
  shelved: true,
  hub: "https://sjk.dfox.app/v1/donations/goal",
  url: "https://ko-fi.com/soljk",
  goal: 200,
  raised: 0,
  monthly: true,
};

/** Escape text for safe insertion as HTML. */
function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  })[c]);
}

/** Inline markdown on already-escaped text: code, bold, links, bare URLs. */
function inline(escaped) {
  const codes = [];
  let text = escaped.replace(/`([^`]+)`/g, (_, code) => {
    codes.push(`<code>${code}</code>`);
    return `\u0000${codes.length - 1}\u0000`;
  });
  text = text
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2">$1</a>')
    .replace(/(^|[\s(])(https?:\/\/[^\s<)]+)/g, (_, lead, url) => {
      const label = url.replace(/^https:\/\/github\.com\/[^/]+\/[^/]+\/pull\//, "#");
      return `${lead}<a href="${url}">${label}</a>`;
    });
  return text.replace(/\u0000(\d+)\u0000/g, (_, i) => codes[Number(i)]);
}

/** The `<img>` tags of a raw HTML line, rebuilt from their https src, alt and width. */
function htmlImages(raw) {
  const images = [];
  for (const [tag] of raw.matchAll(/<img\b[^>]*>/gi)) {
    const attr = (name) => (tag.match(new RegExp(`\\s${name}\\s*=\\s*"([^"]*)"`, "i")) || [])[1];
    const src = attr("src");
    if (!src || !/^https:\/\//i.test(src)) continue;
    const width = /^\d+$/.test(attr("width") || "") ? ` width="${attr("width")}"` : "";
    images.push(`<img src="${escapeHtml(src)}" alt="${escapeHtml(attr("alt") || "")}"${width}>`);
  }
  return images;
}

/** A small markdown subset for release notes: headings, lists, paragraphs.
 * A line of raw HTML keeps only its images; any other markup is dropped. */
function renderMarkdown(source) {
  const out = [];
  let list = false;
  let paragraph = [];
  const flush = () => {
    if (paragraph.length) out.push(`<p>${inline(paragraph.join(" "))}</p>`);
    paragraph = [];
  };
  const closeList = () => {
    if (list) out.push("</ul>");
    list = false;
  };
  for (const raw of String(source || "").replace(/\r\n?/g, "\n").split("\n")) {
    if (raw.trim().startsWith("<")) {
      flush(); closeList();
      const images = htmlImages(raw);
      if (images.length) out.push(`<p class="release-image">${images.join(" ")}</p>`);
      continue;
    }
    const line = escapeHtml(raw.trimEnd());
    const heading = line.match(/^#{1,6}\s+(.*)$/);
    const item = line.match(/^\s*[-*]\s+(.*)$/);
    if (heading) {
      flush(); closeList();
      out.push(`<h4>${inline(heading[1])}</h4>`);
    } else if (item) {
      flush();
      if (!list) { out.push("<ul>"); list = true; }
      out.push(`<li>${inline(item[1])}</li>`);
    } else if (!line.trim()) {
      flush(); closeList();
    } else {
      closeList();
      paragraph.push(line.trim());
    }
  }
  flush(); closeList();
  return out.join("\n");
}

/** The newest published, non-prerelease release, or the newest of any kind. */
function latestRelease(releases) {
  const published = releases.filter((r) => !r.draft);
  return published.find((r) => !r.prerelease) || published[0] || null;
}

/** Download asset for a platform ("windows-x64", "linux-x64" or "macos-arm64"). */
function assetFor(release, platform) {
  return (release.assets || []).find((a) => a.name.endsWith(`-${platform}.zip`)) || null;
}

function versionOf(release) {
  return (release.tag_name || "").replace(/^sjk-v/, "") || release.name || "";
}

function guessPlatform() {
  const agent = `${navigator.userAgent} ${navigator.platform || ""}`;
  return /Linux|X11/i.test(agent) && !/Android/i.test(agent) ? "linux-x64" : "windows-x64";
}

function enhanceDownload(release) {
  const button = document.getElementById("download");
  const files = document.getElementById("release-files");
  if (!button || !release) return;
  const version = versionOf(release);
  const platform = guessPlatform();
  const asset = assetFor(release, platform);
  const system = platform === "linux-x64" ? "Linux" : "Windows";
  // The button keeps its one word; the line under it names the build.
  button.href = asset ? asset.browser_download_url : release.html_url;
  const line = document.getElementById("download-line");
  if (line) line.textContent = asset ? `Sol JK ${version} for ${system} x64` : `Sol JK ${version}`;
  const name = document.getElementById("release-name");
  if (name) name.innerHTML = `<a href="${escapeHtml(release.html_url)}">${escapeHtml(release.name || version)}</a>`;
  const note = document.getElementById("release-note");
  if (note && release.published_at) {
    note.textContent = `Released ${euDate(release.published_at)} · needs your own copy of Jedi Academy`;
  }
  const corner = document.getElementById("version");
  if (corner) corner.textContent = `Sol JK ${version}`;
  if (files) {
    const labels = { "windows-x64": "Windows x64", "linux-x64": "Linux x64", "macos-arm64": "macOS Apple Silicon (experimental)" };
    const links = Object.keys(labels)
      .map((p) => [p, assetFor(release, p)])
      .filter(([, a]) => a)
      .map(([p, a]) => `<a href="${escapeHtml(a.browser_download_url)}">${labels[p]}</a>`);
    links.push(`<a href="${escapeHtml(release.html_url)}">all files &amp; checksums</a>`);
    files.innerHTML = links.join(" · ");
    files.hidden = false;
  }
}

/** A whole number of euros as SJK's site writes them: "1 200 €". */
function euros(amount) {
  const whole = Math.max(0, Math.round(Number(amount) || 0));
  const narrowSpace = String.fromCharCode(0x202f);
  const noBreak = String.fromCharCode(0xa0);
  return `${String(whole).replace(/\B(?=(\d{3})+(?!\d))/g, narrowSpace)}${noBreak}€`;
}

/** What the donation card shows for `donation`: the amount line, the bar's fill
 * (0 to 1) and the link, or null while the link is not an https address. */
function donationState(donation) {
  const goal = Math.max(1, Math.round(Number(donation.goal) || 0));
  const raised = Math.max(0, Math.round(Number(donation.raised) || 0));
  const url = /^https:\/\/\S+$/.test(String(donation.url || "")) ? donation.url : null;
  const period = donation.monthly === false ? "" : " this month";
  return {
    amount: `${euros(raised)} of ${euros(goal)}${period}`,
    fill: Math.min(1, raised / goal),
    goal,
    raised,
    url,
  };
}

/** The hub's goal answer as DONATION's fields, or null when it is not one. */
function donationFromHub(answer) {
  if (!answer || !(Number(answer.goal_cents) > 0)) return null;
  const url = /^https:\/\/\S+$/.test(String(answer.url || "")) ? answer.url : DONATION.url;
  return {
    url,
    goal: Number(answer.goal_cents) / 100,
    raised: Math.max(0, Number(answer.raised_cents) || 0) / 100,
    monthly: answer.period !== "total",
  };
}

/** The donation card from DONATION at once, then from the hub's live goal. */
async function setUpDonation() {
  if (DONATION.shelved) return;
  showDonation(DONATION);
  if (!DONATION.hub || typeof fetch !== "function") return;
  const abort = typeof AbortController === "function" ? new AbortController() : null;
  const timer = abort ? setTimeout(() => abort.abort(), 6000) : null;
  try {
    const response = await fetch(DONATION.hub, abort ? { signal: abort.signal } : {});
    if (!response.ok) throw new Error(`hub ${response.status}`);
    const live = donationFromHub(await response.json());
    if (live) showDonation(live);
  } catch (error) {
    // Keep DONATION's values; the link still works.
    console.info("Donation goal unavailable:", error);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

/** Show `donation` on the card: the goal, the bar and the button. */
function showDonation(donation) {
  const amount = document.getElementById("donation-amount");
  const bar = document.getElementById("donation-bar");
  const note = document.getElementById("donation-note");
  const link = document.getElementById("donation-link");
  if (!amount || !bar || !link) return;
  const state = donationState(donation);
  amount.textContent = state.amount;
  bar.hidden = false;
  bar.setAttribute("aria-valuemin", "0");
  bar.setAttribute("aria-valuemax", String(state.goal));
  bar.setAttribute("aria-valuenow", String(Math.min(state.raised, state.goal)));
  bar.style.setProperty("--fill", String(state.fill));
  if (note) note.hidden = donation.monthly === false;
  link.hidden = false;
  if (state.url) {
    link.href = state.url;
    link.rel = "noopener";
    link.textContent = "Donate on Ko-fi";
    link.classList.remove("waiting");
    link.removeAttribute("aria-disabled");
  } else {
    link.textContent = "Donation link coming soon";
    link.classList.add("waiting");
    link.setAttribute("aria-disabled", "true");
  }
}

/** A date as SJK writes them for people: dd/mm/yyyy, in the reader's time zone. */
function euDate(iso) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  const two = (n) => String(n).padStart(2, "0");
  return `${two(date.getDate())}/${two(date.getMonth() + 1)}/${date.getFullYear()}`;
}

function renderReleases(releases) {
  const target = document.getElementById("releases");
  if (!target) return;
  const shown = releases.filter((r) => !r.draft).slice(0, 5);
  if (!shown.length) {
    target.innerHTML = `<p class="muted">No release yet — the first one is on its way.
      Meanwhile you can build SJK from source (see Install), or follow
      <a href="${RELEASES_PAGE}">the releases page</a>.</p>`;
    return;
  }
  target.innerHTML = shown.map((r) => {
    const when = r.published_at ? `<time datetime="${escapeHtml(r.published_at)}">${euDate(r.published_at)}</time>` : "";
    const title = escapeHtml(r.name || r.tag_name);
    return `<article class="release">
      <header><h3><a href="${escapeHtml(r.html_url)}">${title}</a></h3>${when}</header>
      <div class="release-body">${renderMarkdown(r.body)}</div>
    </article>`;
  }).join("\n") + `<p class="muted"><a href="${RELEASES_PAGE}">All releases</a></p>`;
}

async function loadReleases() {
  try {
    const response = await fetch(RELEASES_API, { headers: { Accept: "application/vnd.github+json" } });
    if (!response.ok) throw new Error(`GitHub API ${response.status}`);
    const releases = await response.json();
    if (!Array.isArray(releases)) throw new Error("unexpected response");
    renderReleases(releases);
    enhanceDownload(latestRelease(releases));
  } catch (error) {
    // Keep the static links; they always work.
    console.info("Release information unavailable:", error);
  }
}

async function loadGallery() {
  const section = document.getElementById("screenshots");
  const gallery = document.getElementById("gallery");
  const dialog = document.getElementById("lightbox");
  if (!section || !gallery) return;
  let shots;
  try {
    const response = await fetch("screenshots/manifest.json", { cache: "no-cache" });
    if (!response.ok) return;
    shots = (await response.json()).filter((s) => s && typeof s.file === "string");
  } catch {
    return;
  }
  if (!shots.length) return;
  gallery.innerHTML = shots.map((s, i) => `<button type="button" class="shot" data-index="${i}">
      <img src="screenshots/${encodeURI(s.file)}" alt="${escapeHtml(s.caption || "SJK screenshot")}" loading="lazy" decoding="async">
      ${s.caption ? `<span>${escapeHtml(s.caption)}</span>` : ""}
    </button>`).join("\n");
  section.hidden = false;
  document.querySelectorAll("[data-gallery-link]").forEach((el) => { el.hidden = false; });

  if (!dialog || typeof dialog.showModal !== "function") {
    gallery.addEventListener("click", (event) => {
      const button = event.target.closest(".shot");
      if (button) window.open(`screenshots/${encodeURI(shots[Number(button.dataset.index)].file)}`, "_blank", "noopener");
    });
    return;
  }
  const image = document.getElementById("lightbox-image");
  const caption = document.getElementById("lightbox-caption");
  let current = 0;
  const show = (index) => {
    current = (index + shots.length) % shots.length;
    image.src = `screenshots/${encodeURI(shots[current].file)}`;
    image.alt = shots[current].caption || "SJK screenshot";
    caption.textContent = shots[current].caption || "";
  };
  gallery.addEventListener("click", (event) => {
    const button = event.target.closest(".shot");
    if (!button) return;
    show(Number(button.dataset.index));
    dialog.showModal();
  });
  dialog.addEventListener("click", (event) => {
    const step = event.target.closest("[data-step]");
    if (step) show(current + Number(step.dataset.step));
    else if (event.target.closest("[data-close]") || event.target === dialog) dialog.close();
  });
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "ArrowRight") show(current + 1);
    if (event.key === "ArrowLeft") show(current - 1);
  });
}

const reducedMotion = typeof window !== "undefined"
  && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Behind the home page, shots of the client's camera tour of Yavin Training
 * Grounds, 15 seconds each, crossing through the navy ground. */
function setUpBackdrop() {
  const shots = [...document.querySelectorAll(".backdrop img")];
  if (shots.length < 2 || reducedMotion) return;
  let current = 0;
  setInterval(() => {
    if (document.hidden) return;
    const next = shots[(current + 1) % shots.length];
    const show = () => {
      shots[current].classList.remove("shown");
      next.classList.add("shown");
      current = shots.indexOf(next);
    };
    if (next.dataset.src) {
      next.addEventListener("load", show, { once: true });
      next.src = next.dataset.src;
      delete next.dataset.src;
    } else {
      show();
    }
  }, 15000);
}

/** The top line stays away while the home page fills the screen, and marks
 * the section being read. */
function setUpNav() {
  const nav = document.querySelector(".nav");
  const home = document.querySelector(".home");
  if (!nav || !home || !("IntersectionObserver" in window)) return;
  new IntersectionObserver(([entry]) => {
    nav.classList.toggle("away", entry.intersectionRatio > 0.3);
  }, { threshold: [0, 0.3, 1] }).observe(home);
  const links = new Map([...nav.querySelectorAll("ul a")].map((a) => [a.getAttribute("href").slice(1), a]));
  const spy = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) links.forEach((a, id) => a.classList.toggle("chosen", id === entry.target.id));
    }
  }, { rootMargin: "-40% 0px -55% 0px" });
  links.forEach((_, id) => {
    const section = document.getElementById(id);
    if (section) spy.observe(section);
  });
}

/** What SJK brings, as the client's Settings: one category on show, chosen
 * on the rail by a click or the arrow keys. Without the script all are listed. */
function setUpSettings() {
  const settings = document.querySelector("[data-settings]");
  if (!settings) return;
  const rail = settings.querySelector(".rail");
  const tabs = [...rail.querySelectorAll("a")];
  const cats = tabs.map((a) => document.getElementById(a.getAttribute("href").slice(1)));
  if (cats.some((c) => !c)) return;
  settings.classList.add("tabbed");
  rail.setAttribute("role", "tablist");
  const show = (index, focus) => {
    tabs.forEach((tab, i) => {
      const chosen = i === index;
      tab.classList.toggle("chosen", chosen);
      tab.setAttribute("role", "tab");
      tab.setAttribute("aria-selected", String(chosen));
      tab.tabIndex = chosen ? 0 : -1;
      cats[i].setAttribute("role", "tabpanel");
      cats[i].classList.toggle("shown", chosen);
    });
    if (focus) tabs[index].focus();
  };
  tabs.forEach((tab, i) => {
    tab.addEventListener("click", (event) => {
      event.preventDefault();
      show(i, false);
    });
  });
  rail.addEventListener("keydown", (event) => {
    const at = tabs.indexOf(document.activeElement);
    const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[event.key];
    if (at < 0 || !step) return;
    event.preventDefault();
    show((at + step + tabs.length) % tabs.length, true);
  });
  show(0, false);
}

if (typeof document !== "undefined") {
  document.documentElement.classList.add("js");
  document.addEventListener("DOMContentLoaded", () => {
    setUpBackdrop();
    setUpNav();
    setUpSettings();
    setUpDonation();
    loadReleases();
    loadGallery();
  });
}

if (typeof module !== "undefined") {
  module.exports = {
    escapeHtml, inline, renderMarkdown, latestRelease, assetFor, versionOf, euDate,
    euros, donationState, donationFromHub, DONATION,
  };
}
