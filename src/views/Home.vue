<template>
  <div>
    <h1>News</h1>
    <ul>
      <li v-for="item in items" :key="item.id">{{ item.text }}</li>
    </ul>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { openDB } from "idb";
import CryptoJS from "crypto-js";

const items = ref([]);

const dbPromise = openDB("news-db", 1, {
  upgrade(db) {
    db.createObjectStore("news", { keyPath: "id" });
    db.createObjectStore("settings");
  },
});

async function loadItems() {
  const db = await dbPromise;
  const tx = db.transaction("news");
  const all = await tx.store.getAll();
  items.value = all;
}

async function connectStream() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const keyBytes = pwd
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;

  const resp = await fetch("https://feilongfl-1.gl.srv.us/stream");
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
      items.value.push(news);
    }
  }
}

onMounted(async () => {
  await loadItems();
  connectStream();
});
</script>
