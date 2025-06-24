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

onMounted(async () => {
  const db = await dbPromise;
  const saved = await db.get("settings", "password");
  if (saved) pwd.value = saved;
  const url = await db.get("settings", "streamUrl");
  if (url) streamUrl.value = url;
});

async function save() {
  const db = await dbPromise;
  const tx = db.transaction("settings", "readwrite");
  tx.store.put(pwd.value, "password");
  tx.store.put(streamUrl.value, "streamUrl");
  await tx.done;
  alert("saved");
}
</script>
