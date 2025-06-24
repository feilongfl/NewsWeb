import { openDB } from "idb";

export const dbPromise = openDB("news-db", 1, {
  upgrade(db) {
    if (!db.objectStoreNames.contains("news")) {
      db.createObjectStore("news", { keyPath: "id" });
    }
    if (!db.objectStoreNames.contains("settings")) {
      db.createObjectStore("settings");
    }
  },
});
