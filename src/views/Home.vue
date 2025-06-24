<template>
  <div class="container">
    <div v-for="item in parsedItems" :key="item.id" class="card mb-4">
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
const parsedItems = computed(() =>
  visibleItems.value.map((item) => {
    try {
      const data = JSON.parse(item.text);
      return { id: item.id, ...data };
    } catch {
      return { id: item.id, raw: item.text };
    }
  }),
);

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
  if (keyBytes) {
    for (const item of latest) {
      try {
        const ciphertext = CryptoJS.enc.Base64.parse(item.text);
        const plain = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
          mode: CryptoJS.mode.ECB,
          padding: CryptoJS.pad.Pkcs7,
        }).toString(CryptoJS.enc.Utf8);
        if (plain) {
          item.text = plain;
          tx.store.put(item);
        }
      } catch {
        // not encrypted or failed to decrypt
      }
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
  const keyBytes = pwd
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;

  const resp = await fetch(url);
  const reader = resp.body.getReader();
  const decoder = new TextDecoder();
  let buf = "";
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
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
        } catch {
          continue;
        }
      }
      const news = { id: Date.now() + Math.random(), text };
      const txw = db.transaction("news", "readwrite");
      txw.store.add(news);
      items.value.unshift(news);
      if (items.value.length > 10) items.value.pop();
    }
  }
}

onMounted(async () => {
  await loadItems();
  connectStream();
});
</script>
