<script setup lang="ts">
import { onMounted, ref } from 'vue';
import Database from '@silvermine/tauri-plugin-sqlite';
import type { KeysetColumn, KeysetPage, SqlValue } from '@silvermine/tauri-plugin-sqlite';
import './style.css';
import ControlPanel from './components/ControlPanel.vue';
import VirtualizedList from './components/VirtualizedList.vue';
import PerformancePanel from './components/PerformancePanel.vue';

export interface Post {
   id: number;
   title: string;
   category: string;
   score: number;
   content: string;
}

export interface TimingEntry {
   page: number;
   method: string;
   timeMs: number;
   rows: number;
}

const DB_PATH = 'pagination-demo.db',
      PAGE_SIZE = 100,
      CATEGORIES = ['tech', 'science', 'sports', 'music', 'art', 'food', 'travel', 'health'];

const LOREM_SENTENCES = [
   'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
   'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.',
   'Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.',
   'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
   'Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium.',
   'Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores.',
   'Neque porro quisquam est qui dolorem ipsum quia dolor sit amet consectetur adipisci velit.',
   'Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam.',
   'Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur.',
   'At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti.',
];

const generateContent = (seed: number): string => {
   const count = 3 + (seed % 3),
         sentences: string[] = [];

   for (let i = 0; i < count; i++) {
      sentences.push(LOREM_SENTENCES[(seed + i * 7) % LOREM_SENTENCES.length]);
   }

   return sentences.join(' ');
};

const keyset: KeysetColumn[] = [
   { name: 'category', direction: 'asc' },
   { name: 'score', direction: 'desc' },
   { name: 'id', direction: 'asc' },
];

const posts = ref<Post[]>([]),
      timings = ref<TimingEntry[]>([]),
      ready = ref(false),
      seeding = ref(false),
      seeded = ref(false),
      loading = ref(false),
      hasMore = ref(false),
      method = ref('keyset'),
      error = ref(''),
      pageNumber = ref(0),
      dataBytes = ref(0);

let db: Database | undefined,
    currentCursor: SqlValue[] | undefined,
    loadGeneration = 0;

const seedData = async (count: number): Promise<void> => {
   if (!db) { return; }
   seeding.value = true;
   error.value = '';

   try {
      await db.execute('DELETE FROM posts');

      const batchSize = 1000,
            batches = Math.ceil(count / batchSize);

      for (let batch = 0; batch < batches; batch++) {
         const rowsInBatch = Math.min(batchSize, count - (batch * batchSize)),
               statements: Array<[string, SqlValue[]]> = [];

         for (let i = 0; i < rowsInBatch; i++) {
            const rowNum = (batch * batchSize) + i + 1,
                  category = CATEGORIES[rowNum % CATEGORIES.length],
                  score = Math.floor(Math.random() * 1000),
                  title = `Post #${rowNum}: ${category} article`,
                  content = generateContent(rowNum);

            statements.push([
               'INSERT INTO posts (title, category, score, content) VALUES ($1, $2, $3, $4)',
               [title, category, score, content],
            ]);
         }

         await db.executeTransaction(statements);
      }

      seeded.value = true;
   } catch (err) {
      error.value = `Seed failed: ${String(err)}`;
   } finally {
      seeding.value = false;
   }
};

const formatBytes = (bytes: number): string => {
   if (bytes < 1024) { return `${bytes} B`; }
   if (bytes < 1024 * 1024) { return `${(bytes / 1024).toFixed(1)} KB`; }
   return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const resetPages = (): void => {
   loadGeneration++;
   loading.value = false;
   posts.value = [];
   timings.value = [];
   pageNumber.value = 0;
   currentCursor = undefined;
   hasMore.value = false;
   dataBytes.value = 0;
};

const loadNextPage = async (): Promise<void> => {
   if (!db || loading.value) { return; }
   loading.value = true;
   error.value = '';

   const generation = loadGeneration;

   try {
      const start = performance.now();

      let newRows: Post[] = [];

      if (method.value === 'keyset') {
         let builder = db.fetchPage<Post>(
            'SELECT id, title, category, score, content FROM posts',
            [],
            keyset,
            PAGE_SIZE
         );

         if (currentCursor) {
            builder = builder.after(currentCursor);
         }

         const page: KeysetPage<Post> = await builder;

         if (generation !== loadGeneration) { return; }

         const elapsed = performance.now() - start;

         newRows = page.rows;
         pageNumber.value += 1;
         posts.value = [...posts.value, ...newRows];
         currentCursor = page.nextCursor ?? undefined;
         hasMore.value = page.hasMore;

         timings.value = [...timings.value, {
            page: pageNumber.value,
            method: 'keyset',
            timeMs: elapsed,
            rows: newRows.length,
         }];
      } else if (method.value === 'all') {
         const rows = await db.fetchAll<Post[]>(
            `SELECT id, title, category, score, content FROM posts
             ORDER BY category ASC, score DESC, id ASC`,
            []
         );

         if (generation !== loadGeneration) { return; }

         const elapsed = performance.now() - start;

         newRows = rows;
         pageNumber.value = 1;
         posts.value = newRows;
         hasMore.value = false;
         dataBytes.value = 0;

         timings.value = [{
            page: 1,
            method: 'all',
            timeMs: elapsed,
            rows: newRows.length,
         }];
      }

      const batchBytes = new Blob([JSON.stringify(newRows)]).size;

      dataBytes.value += batchBytes;
   } catch (err) {
      if (generation !== loadGeneration) { return; }
      error.value = `Load failed: ${String(err)}`;
   } finally {
      if (generation === loadGeneration) {
         loading.value = false;
      }
   }
};

const handleMethodChange = (newMethod: string): void => {
   method.value = newMethod;
   resetPages();
   if (newMethod !== 'all') {
      loadNextPage();
   }
};

onMounted(async () => {
   try {
      db = await Database.load(DB_PATH);

      await db.execute(`
         CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            score INTEGER NOT NULL,
            content TEXT NOT NULL,
            createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
         )
      `);

      await db.execute(`
         CREATE INDEX IF NOT EXISTS idx_posts_keyset
         ON posts (category ASC, score DESC, id ASC)
      `);

      // Check if data already exists
      const countRow = await db.fetchOne<{ count: number }>(
         'SELECT COUNT(*) as count FROM posts'
      );

      if (countRow && countRow.count > 0) {
         seeded.value = true;
      }

      ready.value = true;
   } catch (err) {
      error.value = String(err);
   }
});
</script>

<template>
   <div class="app">
      <header class="app-header">
         <h1>SQLite Pagination Demo</h1>
         <p class="app-header-subtitle">
            Keyset pagination vs fetch-all performance with
            <code>@silvermine/tauri-plugin-sqlite</code>
         </p>
      </header>

      <div v-if="error" class="app-error">{{ error }}</div>

      <template v-if="ready">
         <ControlPanel :seeding="seeding" :seeded="seeded"
            :method="method"
            @seed="seedData"
            @update:method="handleMethodChange"
            @reset="resetPages">
         </ControlPanel>

         <div v-if="seeded && method === 'all' && posts.length === 0" class="app-actions">
            <button :disabled="loading" class="app-loadBtn"
               @click="loadNextPage">
               {{ loading ? 'Loading...' : 'Fetch All Rows' }}
            </button>
         </div>

         <div v-if="seeded" class="app-panels">
            <VirtualizedList :posts="posts" :loading="loading"
               :has-more="hasMore"
               :data-bytes="dataBytes"
               :format-bytes="formatBytes"
               :latest-timing="timings.length > 0 ? timings[timings.length - 1] : undefined"
               @load-more="loadNextPage">
            </VirtualizedList>
            <PerformancePanel :timings="timings"
               :method="method">
            </PerformancePanel>
         </div>
      </template>

      <div v-else-if="!error" class="app-loading">
         Connecting to database...
      </div>
   </div>
</template>

<style scoped>
.app {
   max-width: 1100px;
   margin: 0 auto;
   padding: 24px;
}

.app-header {
   margin-block-end: 24px;
}

.app-header h1 {
   font-size: 22px;
   font-weight: 700;
   margin-block-end: 4px;
}

.app-header-subtitle {
   font-size: 13px;
   color: var(--color-text-secondary);
}

.app-header-subtitle code {
   font-family: var(--font-mono);
   font-size: 12px;
   background-color: var(--color-bg-secondary);
   padding: 1px 5px;
   border-radius: var(--radius-sm);
}

.app-error {
   padding: 12px 16px;
   background-color: #ffeceb;
   color: #ff3b30;
   border-radius: var(--radius);
   margin-block-end: 16px;
   font-family: var(--font-mono);
   font-size: 13px;
}

.app-actions {
   margin-block-end: 16px;
}

.app-loadBtn {
   background-color: var(--color-accent);
   border-color: var(--color-accent);
   color: #fff;
}

.app-loadBtn:hover:not(:disabled) {
   background-color: var(--color-accent-hover);
}

.app-panels {
   display: grid;
   grid-template-columns: 2fr 1fr;
   gap: 16px;
}

.app-loading {
   color: var(--color-text-secondary);
   text-align: center;
   padding: 48px 0;
}

@media (max-width: 800px) {
   .app {
      padding: 12px;
   }

   .app-panels {
      grid-template-columns: 1fr;
   }
}
</style>
