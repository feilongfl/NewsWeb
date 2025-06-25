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
              >{{ item.title }}</a>
              <span v-else>{{ item.title }}</span>
            </p>
            <span v-if="item.type" class="card-header-icon">
              {{ item.type }}
            </span>
          </header>

          <div class="card-content">
            <div class="content" v-if="item.content">{{ item.content }}</div>
            <div class="content" v-else>{{ item.raw }}</div>

            <footer class="card-footer" v-if="item.displayTime">
              <div class="card-footer-item">{{ item.displayTime }}</div>
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

/* === 列表与分页控制 === */
const items = ref([]);
const pageSize = ref(10);
const displayCount = ref(pageSize.value);

/* 当前时间，每 60 秒自动更新一次，用于计算相对时间 */
const now = ref(dayjs());
setInterval(() => {
  now.value = dayjs();
}, 60_000);

/**
 * 只保留一个计算属性，直接把原始 items 切片并解析：
 * - 解密 / 反序列化
 * - 生成 displayTime
 */
const displayItems = computed(() =>
  items.value.slice(0, displayCount.value).map((item) => {
    /* ---------- 解析文本 ---------- */
    let obj;
    if (item.text) {
      try {
        obj = { id: item.id, ...JSON.parse(item.text) };
      } catch {
        obj = { id: item.id, raw: item.text };
      }
    } else {
      obj = { ...item };
    }

    /* ---------- 处理时间 ---------- */
    if (obj.time) {
      const t = dayjs.utc(obj.time.replace("_", " ") + "+08:00").local();
      const diffSec = now.value.diff(t, "second");
      if (diffSec < 60) {
        obj.displayTime = `${diffSec} 秒前`;
      } else if (diffSec < 3_600) {
        obj.displayTime = `${Math.floor(diffSec / 60)} 分钟前`;
      } else if (diffSec < 86_400) {
        obj.displayTime = `${Math.floor(diffSec / 3_600)} 小时前`;
      } else if (diffSec < 2_592_000) {
        obj.displayTime = `${Math.floor(diffSec / 86_400)} 天前`;
      } else {
        obj.displayTime = t.format("YYYY-MM-DD HH:mm");
      }
    }

    return obj;
  }),
);

/* ========== 通知 & 分页 ========== */
function sendNotification(item) {
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

/* ========== IndexedDB: 初始加载 + 数据解密 ========== */
async function loadItems() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const keyBytes = pwd ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8)) : null;

  const tx = db.transaction("news", "readwrite");
  const all = await tx.store.getAll();
  all.sort((a, b) => b.id - a.id);

  for (const item of all) {
    if (item.text) {
      let text = item.text;
      if (keyBytes) {
        try {
          const ciphertext = CryptoJS.enc.Base64.parse(text);
          text = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
            mode: CryptoJS.mode.ECB,
            padding: CryptoJS.pad.Pkcs7,
          }).toString(CryptoJS.enc.Utf8);
        } catch { /* 解密失败保持原样 */ }
      }
      try {
        Object.assign(item, JSON.parse(text));
      } catch {
        item.raw = text;
      }
      delete item.text;
      tx.store.put(item);        // 存回解密后的版本
    }
  }
  items.value = all;
  await tx.done;
}

/* ========== 服务器推流连接 ========== */
async function connectStream() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const url = (await db.get("settings", "streamUrl")) || "https://feilongfl-1.gl.srv.us/stream";
  const delay = (await db.get("settings", "reconnectDelay")) || 5000;
  const keyBytes = pwd ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8)) : null;

  let reader;
  try {
    reader = (await fetch(url)).body.getReader();
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
      const lines = buf.split(/\r?\n/);
      buf = lines.pop();           // 保留最后一行残留

      for (const line of lines) {
        if (!line.startsWith("data:")) continue;
        const payload = line.slice(5).trim();
        if (!payload) continue;

        /* 解密 */
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

        /* 解析 + 入库 + 刷新列表 */
        const news = { id: Date.now() + Math.random() };
        try {
          Object.assign(news, JSON.parse(text));
        } catch {
          news.raw = text;
        }
        const txw = db.transaction("news", "readwrite");
        txw.store.add(news);
        items.value.unshift(news);       // 最新的放最前
        sendNotification(news);
      }
    }
  } catch (e) {
    console.error("stream read error", e);
  }
  setTimeout(connectStream, delay);
}

/* ========== 初始化 ========== */
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
.item-wrapper {
  padding-bottom: 1rem;   /* 原来的 mb-4 */
  box-sizing: border-box;
}
</style>
