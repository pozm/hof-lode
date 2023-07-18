<script lang="ts">
  import { createEventDispatcher } from "svelte";
    import IcoCheck from "./IcoCheck.svelte";
import type { poll_option_housing } from "./server_def";
  import { fade } from "svelte/transition";
    export let option : poll_option_housing
    let hovered = false;
    export let selected = false
    export let bad = false;

    let color = ""
    let color2 = ""
    const sel_dispatch = createEventDispatcher();

    $: {
        if (hovered) {color = "bg-neutral-100 text-black";}
        else if (bad) {color = "bg-red-500"; color2 = "border-red-500"}
        else if (selected) {color = "bg-emerald-400 text-black"; color2 = "border-emerald-400"}
        else {color =  "bg-neutral-800"; color2="border-neutral-800" }
    }

    function send_select_update() {
        sel_dispatch("notify",option.option_id)
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div on:click={send_select_update} tabindex="0" role="checkbox" aria-checked={selected} on:mouseenter={() => {hovered = true}} on:mouseleave={() => {hovered = false}} style="background-image : url({option.image})" class="relative mt-4 select-none {selected && "z-50"} bg-center bg-cover cursor-pointer w-72 aspect-[5/4] hover:scale-150 rounded-xl {color2} transition-all hover:border-neutral-100 border-2" >
    <!-- <img class="absolute w-full h-full object-cover rounded-lg -z-10"  src={option.image} alt={option.name} /> -->
    <p class="absolute z-10 rounded-br-xl rounded-tl-xl -top-0.5 -left-0.5 p-2 {color} transition-colors" >{option.name}</p>
    {#if selected}
        <div transition:fade={{duration:100}}  class="absolute -bottom-0.5 -right-0.5" > 
            <IcoCheck clzz="w-8 h-8 {color} rounded-br-xl rounded-tl-xl" />
        </div>

    {/if}
</div>