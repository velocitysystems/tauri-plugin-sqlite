<script setup lang="ts">
import { computed } from 'vue';
import type { TimingEntry } from '../App.vue';

const props = defineProps({
   timings: {
      type: Array as () => TimingEntry[],
      required: true,
   },
   method: {
      type: String,
      required: true,
   },
});

interface Summary {
   pages: number;
   avg: number;
   min: number;
   max: number;
   last: number;
   trend: 'Stable' | 'Degrading' | '—';
}

const summary = computed<Summary>(() => {
   const t = props.timings;

   if (t.length === 0) {
      return { pages: 0, avg: 0, min: 0, max: 0, last: 0, trend: '—' };
   }

   const times = t.map((e) => e.timeMs),
         total = times.reduce((a, b) => a + b, 0),
         avg = total / times.length,
         min = Math.min(...times),
         max = Math.max(...times),
         last = times[times.length - 1];

   let trend: Summary['trend'] = '—';

   if (t.length >= 10) {
      const firstFive = times.slice(0, 5),
            lastFive = times.slice(-5),
            avgFirst = firstFive.reduce((a, b) => a + b, 0) / firstFive.length,
            avgLast = lastFive.reduce((a, b) => a + b, 0) / lastFive.length;

      trend = avgLast > avgFirst * 1.5 ? 'Degrading' : 'Stable';
   }

   return { pages: t.length, avg, min, max, last, trend };
});
</script>

<template>
   <section class="performancePanel">
      <h2 class="performancePanel-title">
         Performance ({{ method }})
      </h2>

      <div v-if="summary.pages > 0" class="performancePanel-stats">
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Pages</span>
            <span class="performancePanel-stats-value">{{ summary.pages }}</span>
         </div>
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Avg</span>
            <span class="performancePanel-stats-value">
               {{ summary.avg.toFixed(1) }} ms
            </span>
         </div>
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Min</span>
            <span class="performancePanel-stats-value">
               {{ summary.min.toFixed(1) }} ms
            </span>
         </div>
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Max</span>
            <span class="performancePanel-stats-value">
               {{ summary.max.toFixed(1) }} ms
            </span>
         </div>
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Last</span>
            <span class="performancePanel-stats-value">
               {{ summary.last.toFixed(1) }} ms
            </span>
         </div>
         <div class="performancePanel-stats-card">
            <span class="performancePanel-stats-label">Trend</span>
            <span class="performancePanel-stats-value"
               :class="{
                  'performancePanel-stats-value--stable': summary.trend === 'Stable',
                  'performancePanel-stats-value--degrading': summary.trend === 'Degrading',
               }">
               {{ summary.trend }}
            </span>
         </div>
      </div>

      <div class="performancePanel-body">
         <table v-if="timings.length > 0"
            class="performancePanel-table">
            <thead>
               <tr>
                  <th>Page</th>
                  <th>Method</th>
                  <th>Time (ms)</th>
                  <th>Rows</th>
               </tr>
            </thead>
            <tbody>
               <tr v-for="entry in timings" :key="entry.page">
                  <td class="performancePanel-table-page">{{ entry.page }}</td>
                  <td>
                     <span class="performancePanel-badge"
                        :class="`performancePanel-badge--${entry.method}`">
                        {{ entry.method }}
                     </span>
                  </td>
                  <td class="performancePanel-table-time">
                     {{ entry.timeMs.toFixed(1) }}
                     <div class="performancePanel-bar">
                        <div class="performancePanel-bar-fill"
                           :class="`performancePanel-bar-fill--${entry.method}`"
                           :style="{ width: `${Math.min(100, (entry.timeMs / summary.max) * 100)}%` }">
                        </div>
                     </div>
                  </td>
                  <td class="performancePanel-table-rows">{{ entry.rows }}</td>
               </tr>
            </tbody>
         </table>
         <p v-else class="performancePanel-empty">
            Load pages to see performance metrics.
         </p>
      </div>
   </section>
</template>

<style scoped>
.performancePanel {
   border: 1px solid var(--color-border);
   border-radius: var(--radius);
   overflow: hidden;
}

.performancePanel-title {
   font-size: 14px;
   font-weight: 600;
   padding: 10px 14px;
   background-color: var(--color-bg-secondary);
   border-block-end: 1px solid var(--color-border);
}

/* Summary stats */
.performancePanel-stats {
   display: grid;
   grid-template-columns: repeat(3, 1fr);
   gap: 1px;
   background-color: var(--color-border);
   border-block-end: 1px solid var(--color-border);
}

.performancePanel-stats-card {
   display: flex;
   flex-direction: column;
   align-items: center;
   gap: 2px;
   padding: 8px 6px;
   background-color: var(--color-bg);
}

.performancePanel-stats-label {
   font-size: 10px;
   font-weight: 600;
   text-transform: uppercase;
   letter-spacing: 0.5px;
   color: var(--color-text-secondary);
}

.performancePanel-stats-value {
   font-family: var(--font-mono);
   font-size: 12px;
   font-weight: 600;
}

.performancePanel-stats-value--stable {
   color: var(--color-keyset);
}

.performancePanel-stats-value--degrading {
   color: var(--color-warning);
}

.performancePanel-body {
   max-height: 500px;
   overflow-y: auto;
}

.performancePanel-table {
   width: 100%;
   border-collapse: collapse;
}

.performancePanel-table th,
.performancePanel-table td {
   padding: 6px 14px;
   text-align: start;
   border-block-end: 1px solid var(--color-border);
   font-size: 13px;
}

.performancePanel-table th {
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

.performancePanel-table-page {
   font-family: var(--font-mono);
   font-size: 12px;
   color: var(--color-text-secondary);
}

.performancePanel-table-time {
   font-family: var(--font-mono);
   font-size: 12px;
}

.performancePanel-table-rows {
   font-family: var(--font-mono);
   font-size: 12px;
   color: var(--color-text-secondary);
}

.performancePanel-badge {
   font-size: 11px;
   font-weight: 600;
   text-transform: uppercase;
   padding: 1px 6px;
   border-radius: var(--radius-sm);
}

.performancePanel-badge--keyset {
   color: var(--color-keyset);
   background-color: var(--color-keyset-bg);
}

.performancePanel-badge--all {
   color: var(--color-all);
   background-color: var(--color-all-bg);
}

/* Mini timing bars */
.performancePanel-bar {
   height: 3px;
   margin-block-start: 4px;
   background-color: var(--color-bg-secondary);
   border-radius: 2px;
   overflow: hidden;
}

.performancePanel-bar-fill {
   height: 100%;
   border-radius: 2px;
   transition: width 0.3s ease;
}

.performancePanel-bar-fill--keyset {
   background-color: var(--color-keyset);
}

.performancePanel-bar-fill--all {
   background-color: var(--color-all);
}

.performancePanel-empty {
   padding: 32px 14px;
   text-align: center;
   color: var(--color-text-secondary);
   font-size: 13px;
}
</style>
