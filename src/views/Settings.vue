<template>
  <div class="container">
    <h1 class="title">Settings</h1>
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
import { openDB } from "idb";

const pwd = ref("");
const dbPromise = openDB("news-db", 1, {
  upgrade(db) {
    db.createObjectStore("news", { keyPath: "id" });
    db.createObjectStore("settings");
  },
});

onMounted(async () => {
  const db = await dbPromise;
  const saved = await db.get("settings", "password");
  if (saved) pwd.value = saved;
});

async function save() {
  const db = await dbPromise;
  const tx = db.transaction("settings", "readwrite");
  tx.store.put(pwd.value, "password");
  await tx.done;
  alert("saved");
}
</script>
