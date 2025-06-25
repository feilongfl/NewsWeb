<template>
  <div class="container">
    <h1 class="title">Settings</h1>
    <div class="field">
      <label class="label">Stream URL</label>
      <div class="control">
        <input class="input" v-model="streamUrl" type="text" />
      </div>
    </div>
    <div class="field">
      <label class="label">Password</label>
      <div class="control">
        <input class="input" v-model="pwd" type="text" />
      </div>
    </div>
    <div class="field">
      <label class="label">Reconnect Delay (ms)</label>
      <div class="control">
        <input class="input" v-model.number="reconnectDelay" type="number" />
      </div>
    </div>
    <div class="field">
      <label class="label">Items Per Page</label>
      <div class="control">
        <input class="input" v-model.number="pageSize" type="number" min="1" />
      </div>
    </div>
    <div class="field">
      <div class="control">
        <button class="button is-primary" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { dbPromise } from "../db";

const pwd = ref("");
const streamUrl = ref("https://feilongfl-1.gl.srv.us/stream");
const reconnectDelay = ref(5000);
const pageSize = ref(10);

onMounted(async () => {
  const db = await dbPromise;
  const saved = await db.get("settings", "password");
  if (saved) pwd.value = saved;
  const url = await db.get("settings", "streamUrl");
  if (url) streamUrl.value = url;
  const delay = await db.get("settings", "reconnectDelay");
  if (delay) reconnectDelay.value = delay;
  const ps = await db.get("settings", "pageSize");
  if (ps) pageSize.value = ps;
});

async function save() {
  const db = await dbPromise;
  const tx = db.transaction("settings", "readwrite");
  tx.store.put(pwd.value, "password");
  tx.store.put(streamUrl.value, "streamUrl");
  tx.store.put(Number(reconnectDelay.value), "reconnectDelay");
  tx.store.put(Number(pageSize.value), "pageSize");
  await tx.done;
  alert("saved");
}
</script>
