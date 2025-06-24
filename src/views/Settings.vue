<template>
  <div>
    <h1>Settings</h1>
    <label
      >Password
      <input v-model="pwd" type="text" />
    </label>
    <button @click="save">Save</button>
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
