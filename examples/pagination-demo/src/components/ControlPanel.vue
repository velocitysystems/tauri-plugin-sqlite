<script setup lang="ts">
defineProps({
   seeding: {
      type: Boolean,
      required: true,
   },
   seeded: {
      type: Boolean,
      required: true,
   },
   method: {
      type: String,
      required: true,
   },
});

const emit = defineEmits<{
   seed: [count: number];
   'update:method': [method: string];
   reset: [];
}>();

const handleSeed = (event: Event): void => {
   const form = (event.target as HTMLElement).closest('form');

   if (!form) { return; }

   const input = form.querySelector('input[name="rowCount"]') as HTMLInputElement;
   const count = parseInt(input.value, 10);

   if (count > 0) {
      emit('seed', count);
   }
};
</script>

<template>
   <section class="controlPanel">
      <h2 class="controlPanel-title">Controls</h2>
      <div class="controlPanel-body">
         <form class="controlPanel-seedForm"
            @submit.prevent="handleSeed">
            <label class="controlPanel-label">
               Rows to seed:
               <input type="number" name="rowCount"
                  :value="50000" min="1000" step="1000"
                  :disabled="seeding" />
            </label>
            <button type="submit"
               :disabled="seeding"
               class="controlPanel-btn controlPanel-btn--seed">
               {{ seeding ? 'Seeding...' : 'Seed Data' }}
            </button>
         </form>

         <div v-if="seeded" class="controlPanel-methodToggle">
            <span class="controlPanel-label">Method:</span>
            <button :class="['controlPanel-btn', { 'controlPanel-btn--active': method === 'keyset' }]"
               @click="emit('update:method', 'keyset')">
               Keyset
            </button>
            <button :class="['controlPanel-btn', { 'controlPanel-btn--active': method === 'all' }]"
               @click="emit('update:method', 'all')">
               All
            </button>
            <button class="controlPanel-btn controlPanel-btn--reset"
               @click="emit('reset')">
               Reset
            </button>
         </div>
      </div>
   </section>
</template>

<style scoped>
.controlPanel {
   border: 1px solid var(--color-border);
   border-radius: var(--radius);
   overflow: hidden;
   margin-block-end: 16px;
}

.controlPanel-title {
   font-size: 14px;
   font-weight: 600;
   padding: 10px 14px;
   background-color: var(--color-bg-secondary);
   border-block-end: 1px solid var(--color-border);
}

.controlPanel-body {
   padding: 14px;
   display: flex;
   flex-wrap: wrap;
   align-items: center;
   gap: 16px;
}

.controlPanel-seedForm {
   display: flex;
   align-items: center;
   gap: 8px;
}

.controlPanel-label {
   font-size: 13px;
   display: flex;
   align-items: center;
   gap: 6px;
}

.controlPanel-methodToggle {
   display: flex;
   align-items: center;
   gap: 8px;
}

.controlPanel-btn--seed {
   background-color: var(--color-accent);
   border-color: var(--color-accent);
   color: #fff;
}

.controlPanel-btn--seed:hover:not(:disabled) {
   background-color: var(--color-accent-hover);
}

.controlPanel-btn--active {
   background-color: var(--color-accent);
   border-color: var(--color-accent);
   color: #fff;
}

.controlPanel-btn--reset {
   margin-inline-start: 8px;
}
</style>
