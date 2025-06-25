<template>
  <div class="container">
    <div v-for="item in parsedItems" :key="item.id" class="card mb-4">
      <header class="card-header">
        <p class="card-header-title">
          <a
            v-if="item.link"
            :href="item.link"
            target="_blank"
            rel="noopener"
            >{{ item.title }}</a
          >
          <span v-else>{{ item.title }}</span>
        </p>
        <span v-if="item.type" class="card-header-icon">{{ item.type }}</span>
      </header>
      <div class="card-content">
        <p v-if="item.content">{{ item.content }}</p>
        <p v-else>{{ item.raw }}</p>
        <div v-if="item.time" class="has-text-right is-size-7 mt-2">
          {{ item.time }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { dbPromise } from "../db";
import CryptoJS from "crypto-js";

const items = ref([]);
const visibleItems = computed(() => items.value.slice(0, 10));
const parsedItems = computed(() =>
  visibleItems.value.map((item) => {
    if (item.text) {
      try {
        const data = JSON.parse(item.text);
        return { id: item.id, ...data };
      } catch {
        return { id: item.id, raw: item.text };
      }
    }
    return item;
  }),
);

function sendNotification(item) {
  if (!("Notification" in window)) return;
  if (Notification.permission === "granted") {
    new Notification(item.title || "News", {
      body: item.content || item.raw || "",
    });
  }
}

async function loadItems() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const keyBytes = pwd
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;
  const tx = db.transaction("news", "readwrite");
  const all = await tx.store.getAll();
  all.sort((a, b) => b.id - a.id);
  const latest = all.slice(0, 10);
  for (const item of latest) {
    if (item.text) {
      let text = item.text;
      if (keyBytes) {
        try {
          const ciphertext = CryptoJS.enc.Base64.parse(item.text);
          text = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
            mode: CryptoJS.mode.ECB,
            padding: CryptoJS.pad.Pkcs7,
          }).toString(CryptoJS.enc.Utf8);
        } catch {
          // failed to decrypt, keep original text
        }
      }
      try {
        const data = JSON.parse(text);
        Object.assign(item, data);
      } catch {
        item.raw = text;
      }
      delete item.text;
      tx.store.put(item);
    }
  }
  items.value = latest;
  await tx.done;
}

async function connectStream() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const url =
    (await db.get("settings", "streamUrl")) ||
    "https://feilongfl-1.gl.srv.us/stream";
  const delay = (await db.get("settings", "reconnectDelay")) || 5000;
  const keyBytes = pwd
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;

  let reader;
  try {
    const resp = await fetch(url);
    reader = resp.body.getReader();
  } catch (e) {
    console.error("stream connect error", e);
    setTimeout(connectStream, delay);
    return;
  }

  const decoder = new TextDecoder();
  let buf = "";
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) throw new Error("stream closed");
      buf += decoder.decode(value, { stream: true });
      let lines = buf.split(/\r?\n/);
      buf = lines.pop();
      for (let line of lines) {
        if (!line.startsWith("data:")) continue;
        let payload = line.slice(5).trim();
        if (!payload) continue;
        let text = payload;
        if (keyBytes) {
          try {
            let ciphertext = CryptoJS.enc.Base64.parse(payload);
            text = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
              mode: CryptoJS.mode.ECB,
              padding: CryptoJS.pad.Pkcs7,
            }).toString(CryptoJS.enc.Utf8);
            if (!text) continue;
          } catch {
            continue;
          }
        }
        let news = { id: Date.now() + Math.random() };
        try {
          Object.assign(news, JSON.parse(text));
        } catch {
          news.raw = text;
        }
        const txw = db.transaction("news", "readwrite");
        txw.store.add(news);
        items.value.unshift(news);
        sendNotification(news);
        if (items.value.length > 10) items.value.pop();
      }
    }
  } catch (e) {
    console.error("stream read error", e);
  }
  setTimeout(connectStream, delay);
}

onMounted(async () => {
  if ("Notification" in window && Notification.permission === "default") {
    Notification.requestPermission();
  }
  await loadItems();
  connectStream();
});
</script>
