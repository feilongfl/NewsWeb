<template>
  <div class="container">
    <div v-for="item in parsedItems" :key="item.link" class="card mb-4">
      <header v-if="item.type" class="card-header">
        <p class="card-header-title">{{ item.type }}</p>
      </header>
      <div class="card-content">
        <p v-if="item.title" class="title is-5">
          <a
            v-if="item.link"
            :href="item.link"
            target="_blank"
            rel="noopener"
            >{{ item.title }}</a
          >
          <span v-else>{{ item.title }}</span>
        </p>
        <p v-if="item.content">{{ item.content }}</p>
        <p v-else>{{ item.raw }}</p>
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
const parsedItems = computed(() => visibleItems.value);

async function loadItems() {
  const db = await dbPromise;
  const tx = db.transaction("news");
  const all = await tx.store.getAll();
  all.sort((a, b) => (b.ts || 0) - (a.ts || 0));
  items.value = all.slice(0, 10);
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
        let news;
        try {
          news = JSON.parse(text);
        } catch {
          continue;
        }
        if (!news.link) continue;
        news.ts = Date.now();
        await db.put("news", news);
        items.value.unshift(news);
        if (items.value.length > 10) items.value.pop();
      }
    }
  } catch (e) {
    console.error("stream read error", e);
  }
  setTimeout(connectStream, delay);
}

onMounted(async () => {
  await loadItems();
  connectStream();
});
</script>
