<template>
  <div class="container">
    <DynamicScroller
      class="scroller"
      :items="displayItems"
      :min-item-size="80"
      key-field="id"
      @scroll-end="loadMore"
      v-slot="{ item, active }"
    >
      <!-- 包裹层：用内边距代替外边距，避免高度测量错误 -->
      <DynamicScrollerItem
        :item="item"
        :active="active"
        :size-dependencies="[item.content, item.raw, item.displayTime]"
        tag="div"
        class="item-wrapper"
      >
        <div class="card">
          <header class="card-header">
            <p class="card-header-title">
              <a
                v-if="item.link"
                :href="item.link"
                target="_blank"
                rel="noopener"
              >
                {{ item.title }}
              </a>
              <span v-else>{{ item.title }}</span>
            </p>
            <span v-if="item.type" class="card-header-icon">
              {{ item.type }}
            </span>
          </header>
          <div class="card-content">
            <div class="content" v-if="item.content">{{ item.content }}</div>
            <div class="content" v-else>{{ item.raw }}</div>
            <footer class="card-footer" v-if="item.time">
              <div class="card-footer-item">{{ item.time }}</div>
            </footer>
          </div>
        </div>
      </DynamicScrollerItem>
    </DynamicScroller>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { dbPromise } from "../db";
import CryptoJS from "crypto-js";
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import relativeTime from "dayjs/plugin/relativeTime";
import { DynamicScroller, DynamicScrollerItem } from "vue-virtual-scroller";
import "dayjs/locale/zh-cn";

dayjs.extend(utc);
dayjs.extend(relativeTime);
dayjs.locale("zh-cn");

const items = ref([]);
const pageSize = ref(10);
const displayCount = ref(pageSize.value);

const displayItems = computed(() => items.value.slice(0, displayCount.value));
const parsedItems = computed(() =>
  displayItems.value.map((item) => {
    let obj;
    if (item.text) {
      try {
        const data = JSON.parse(item.text);
        obj = { id: item.id, ...data };
      } catch {
        obj = { id: item.id, raw: item.text };
      }
    } else {
      obj = { ...item };
    }

    if (obj.time) {
      const t = dayjs.utc(obj.time.replace("_", " ") + "+08:00").local();
      const now = dayjs();
      if (now.isSame(t, "day")) {
        obj.displayTime =
          now.diff(t, "second") < 86400 ? t.fromNow() : t.format("HH:mm:ss");
      } else {
        obj.displayTime = t.format("YYYY-MM-DD HH:mm:ss");
      }
    }

    return obj;
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

function loadMore() {
  if (displayCount.value < items.value.length) {
    displayCount.value = Math.min(
      displayCount.value + pageSize.value,
      items.value.length,
    );
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

  for (const item of all) {
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
          /* keep original */
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
  items.value = all;
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
            const ciphertext = CryptoJS.enc.Base64.parse(payload);
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
  const db = await dbPromise;
  const ps = await db.get("settings", "pageSize");
  if (ps) pageSize.value = ps;
  displayCount.value = pageSize.value;
  await loadItems();
  connectStream();
});
</script>

<style scoped>
/* 用内边距替代外边距，防止高度测量误差 */
.item-wrapper {
  padding-bottom: 1rem; /* 相当于原来的 mb-4 */
  box-sizing: border-box;
}
</style>
