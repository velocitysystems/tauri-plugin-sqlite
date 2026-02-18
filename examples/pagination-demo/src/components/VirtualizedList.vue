<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import type { Post, TimingEntry } from '../App.vue';

const props = defineProps({
   posts: {
      type: Array as () => Post[],
      required: true,
   },
   loading: {
      type: Boolean,
      required: true,
   },
   hasMore: {
      type: Boolean,
      required: true,
   },
   dataBytes: {
      type: Number,
      required: true,
   },
   formatBytes: {
      type: Function as unknown as () => (bytes: number) => string,
      required: true,
   },
   latestTiming: {
      type: Object as () => TimingEntry | undefined,
      default: undefined,
   },
});

const emit = defineEmits<{
   'load-more': [];
}>();

const bodyRef = ref<HTMLElement | null>(null),
      toastVisible = ref(false),
      selectedPost = ref<Post | null>(null);

let toastTimer: ReturnType<typeof setTimeout> | undefined;

const handleScroll = (): void => {
   const el = bodyRef.value;

   if (!el || !props.hasMore || props.loading) { return; }
   if (el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
      emit('load-more');
   }
};

const handleKeydown = (e: KeyboardEvent): void => {
   if (e.key === 'Escape' && selectedPost.value) {
      selectedPost.value = null;
   }
};

watch(() => props.latestTiming, (newVal) => {
   if (!newVal) { return; }
   toastVisible.value = true;
   clearTimeout(toastTimer);
   toastTimer = setTimeout(() => { toastVisible.value = false; }, 2500);
});

onMounted(() => {
   document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
   document.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
   <section class="virtualizedList">
      <h2 class="virtualizedList-title">
         Posts ({{ posts.length }} loaded)
         <span v-if="dataBytes > 0" class="virtualizedList-memoryEstimate">
            — ~{{ formatBytes(dataBytes) }}
         </span>
      </h2>
      <Transition name="virtualizedList-toast">
         <div v-if="toastVisible && latestTiming" class="virtualizedList-toast">
            Page {{ latestTiming.page }} loaded — {{ latestTiming.rows }} rows
            in {{ latestTiming.timeMs.toFixed(1) }} ms
         </div>
      </Transition>
      <div ref="bodyRef" class="virtualizedList-body"
         @scroll="handleScroll">
         <table v-if="posts.length > 0"
            class="virtualizedList-table">
            <thead>
               <tr>
                  <th>ID</th>
                  <th>Category</th>
                  <th>Score</th>
                  <th>Title</th>
               </tr>
            </thead>
            <tbody>
               <tr v-for="post in posts" :key="post.id"
                  class="virtualizedList-table-row"
                  @click="selectedPost = post">
                  <td class="virtualizedList-table-id">{{ post.id }}</td>
                  <td>{{ post.category }}</td>
                  <td class="virtualizedList-table-score">{{ post.score }}</td>
                  <td>{{ post.title }}</td>
               </tr>
            </tbody>
         </table>
         <p v-else class="virtualizedList-empty">
            No posts loaded. Seed data first, then load pages.
         </p>
      </div>
      <div v-if="posts.length > 0" class="virtualizedList-footer">
         <span v-if="loading" class="virtualizedList-loading">Loading...</span>
         <span v-else-if="!hasMore" class="virtualizedList-done">
            All rows loaded
         </span>
      </div>

      <Teleport to="body">
         <Transition name="virtualizedList-modal">
            <div v-if="selectedPost" class="virtualizedList-backdrop"
               @click.self="selectedPost = null">
               <div class="virtualizedList-modal">
                  <header class="virtualizedList-modal-header">
                     <h3 class="virtualizedList-modal-title">
                        {{ selectedPost.title }}
                     </h3>
                     <button class="virtualizedList-modal-close"
                        @click="selectedPost = null">
                        &times;
                     </button>
                  </header>
                  <div class="virtualizedList-modal-meta">
                     <span class="virtualizedList-modal-badge">
                        {{ selectedPost.category }}
                     </span>
                     <span class="virtualizedList-modal-stat">
                        Score: {{ selectedPost.score }}
                     </span>
                     <span class="virtualizedList-modal-stat virtualizedList-modal-stat--id">
                        ID: {{ selectedPost.id }}
                     </span>
                  </div>
                  <p class="virtualizedList-modal-content">
                     {{ selectedPost.content }}
                  </p>
               </div>
            </div>
         </Transition>
      </Teleport>
   </section>
</template>

<style scoped>
.virtualizedList {
   border: 1px solid var(--color-border);
   border-radius: var(--radius);
   overflow: hidden;
}

.virtualizedList-title {
   font-size: 14px;
   font-weight: 600;
   padding: 10px 14px;
   background-color: var(--color-bg-secondary);
   border-block-end: 1px solid var(--color-border);
}

.virtualizedList-memoryEstimate {
   font-size: 12px;
   font-weight: 400;
   color: var(--color-text-secondary);
}

.virtualizedList-toast {
   padding: 6px 14px;
   font-size: 12px;
   font-weight: 500;
   color: var(--color-accent);
   background-color: var(--color-bg-secondary);
   border-block-end: 1px solid var(--color-border);
   text-align: center;
}

.virtualizedList-toast-enter-active,
.virtualizedList-toast-leave-active {
   transition: all 0.3s ease;
}

.virtualizedList-toast-enter-from,
.virtualizedList-toast-leave-to {
   opacity: 0;
   max-height: 0;
   padding-block: 0;
}

.virtualizedList-body {
   max-height: 500px;
   overflow-y: auto;
   overflow-x: auto;
}

.virtualizedList-table {
   width: 100%;
   border-collapse: collapse;
}

.virtualizedList-table th,
.virtualizedList-table td {
   padding: 6px 14px;
   text-align: start;
   border-block-end: 1px solid var(--color-border);
   font-size: 13px;
}

.virtualizedList-table th {
   font-size: 11px;
   font-weight: 600;
   text-transform: uppercase;
   letter-spacing: 0.5px;
   color: var(--color-text-secondary);
   background-color: var(--color-bg-secondary);
   position: sticky;
   top: 0;
   z-index: 1;
}

.virtualizedList-table-row {
   cursor: pointer;
   transition: background-color 0.1s ease;
}

.virtualizedList-table-row:hover {
   background-color: var(--color-bg-secondary);
}

.virtualizedList-table-id {
   font-family: var(--font-mono);
   font-size: 12px;
   color: var(--color-text-secondary);
}

.virtualizedList-table-score {
   font-family: var(--font-mono);
   font-size: 12px;
}

.virtualizedList-empty {
   padding: 32px 14px;
   text-align: center;
   color: var(--color-text-secondary);
   font-size: 13px;
}

.virtualizedList-footer {
   padding: 12px 14px;
   text-align: center;
   border-block-start: 1px solid var(--color-border);
   background-color: var(--color-bg-secondary);
}

.virtualizedList-loading {
   font-size: 13px;
   color: var(--color-text-secondary);
}

.virtualizedList-done {
   font-size: 13px;
   color: var(--color-text-secondary);
}

/* Modal */
.virtualizedList-backdrop {
   position: fixed;
   inset: 0;
   background-color: rgba(0, 0, 0, 0.5);
   display: flex;
   align-items: center;
   justify-content: center;
   z-index: 1000;
}

.virtualizedList-modal {
   background-color: var(--color-bg);
   border-radius: var(--radius);
   box-shadow: var(--shadow), 0 8px 32px rgba(0, 0, 0, 0.15);
   width: 90%;
   max-width: 520px;
   max-height: 80vh;
   overflow-y: auto;
}

.virtualizedList-modal-header {
   display: flex;
   align-items: flex-start;
   justify-content: space-between;
   gap: 12px;
   padding: 16px 16px 12px;
   border-block-end: 1px solid var(--color-border);
}

.virtualizedList-modal-title {
   font-size: 15px;
   font-weight: 600;
   line-height: 1.4;
}

.virtualizedList-modal-close {
   flex-shrink: 0;
   width: 28px;
   height: 28px;
   padding: 0;
   font-size: 18px;
   line-height: 1;
   display: flex;
   align-items: center;
   justify-content: center;
   border-radius: var(--radius-sm);
}

.virtualizedList-modal-meta {
   display: flex;
   align-items: center;
   gap: 10px;
   padding: 12px 16px;
   border-block-end: 1px solid var(--color-border);
}

.virtualizedList-modal-badge {
   font-size: 11px;
   font-weight: 600;
   text-transform: uppercase;
   padding: 2px 8px;
   border-radius: var(--radius-sm);
   color: var(--color-accent);
   background-color: var(--color-bg-secondary);
}

.virtualizedList-modal-stat {
   font-size: 12px;
   font-weight: 500;
   color: var(--color-text-secondary);
}

.virtualizedList-modal-stat--id {
   font-family: var(--font-mono);
}

.virtualizedList-modal-content {
   padding: 16px;
   font-size: 13px;
   line-height: 1.6;
   color: var(--color-text);
}

/* Modal transitions */
.virtualizedList-modal-enter-active,
.virtualizedList-modal-leave-active {
   transition: opacity 0.2s ease;
}

.virtualizedList-modal-enter-active .virtualizedList-modal,
.virtualizedList-modal-leave-active .virtualizedList-modal {
   transition: transform 0.2s ease;
}

.virtualizedList-modal-enter-from,
.virtualizedList-modal-leave-to {
   opacity: 0;
}

.virtualizedList-modal-enter-from .virtualizedList-modal {
   transform: scale(0.95);
}

.virtualizedList-modal-leave-to .virtualizedList-modal {
   transform: scale(0.95);
}
</style>
