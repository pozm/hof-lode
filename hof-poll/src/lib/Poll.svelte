<script lang="ts">
    import type {poll} from "./server_def";
    import HousingOptionRender from "./HousingOptionRender.svelte";
  import { createEventDispatcher } from "svelte";

    export let poll : poll;

    function get_options(type:number, id:number) {
        switch (type) {
            case 1: { // housing
                option_renderer = HousingOptionRender
                return fetch("/api/polls/housing/" + id).then(res => res.json());
            } break;

        }
    }
    let submit_res;
    function submit_vote() {
        console.log("submitting vote");
        validate_inputs();
        if (failr != 0) {return};
        let data = {
            player_name: username,
            option_id: which_sel
        }
        submit_res = fetch("/api/polls/housing/" + poll.id,{
            body: JSON.stringify(data),
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            }
        }).then(res => res.json());
    }
    function validate_inputs() {
        if (username == "") {failr = 1}
        else if (which_sel == -1) {failr = 2}
        else {failr = 0};

    }

    let option_renderer;
    let opts = get_options(poll.type, poll.id);
    let which_sel = -1;
    let username = "";
    let failr = 0;

    function update_select(ev) {
        which_sel = ev.detail;
    }
    $: {
        if (failr != 0) {
            if (username != "") {validate_inputs()}
            else if (which_sel != -1) {validate_inputs()}
            // else {failr = 0}
        }
    }

</script>

<div class="bg-neutral-900 border-solid border-neutral-700 border rounded-xl p-4 my-6" >
    <h2 class="text-2xl text-center md:text-left" >{poll.poll_name}</h2>
    {#await opts}
        <p>loading options...</p>
    {:then opts}
        <div class="flex justify-around flex-wrap" >
            
            {#each opts as opt,i}
                <svelte:component bad={failr==2} on:notify={update_select} this={option_renderer} selected={which_sel == opt.option_id} option={opt} ></svelte:component>
            {/each}
        </div>
        <div class="mt-4 flex space-x-4 items-center flex-wrap " >
            <div class="mx-auto sm:mr-0" ></div>
            <div class="w-full sm:w-fit" >

                {#if failr == 1}
                    <p class="text-red-600 " >Please enter a username</p>
                {:else if failr == 2}
                    <p class="text-red-600" >Please select an option</p>
                {/if}
            </div>
            <input required bind:value={username} type="text" placeholder="FFXIV character name" class="py-2 px-4 rounded-xl bg-neutral-700 {failr == 1 && "ring-2 ring-red-600"} focus:outline-none focus:ring-2 focus:ring-red-400" />
            <button on:click={submit_vote} class="py-2 px-4 rounded-xl bg-red-400 text-white" >Submit</button>
        </div>
        {#if submit_res}
            {#await submit_res}
                <p>submitting vote...</p>
            {:then res}
                {#if res.success}
                    <p>vote submitted!</p>
                {:else}
                    <p>Failure to submit : {res.error}</p>
                {/if}
                
            {/await}
        {/if}
    {:catch err}
        <p>oopsies woopsies we made a fucky wucky!</p>
    {/await}


</div>