<script lang="ts">

    import Poll from "./lib/Poll.svelte";

    let polls_fut = fetch("/api/polls").then(res => res.json());

</script>

<main class="container mx-auto" >

    <h1 class="text-4xl" >Currently active polls: </h1>

    {#await polls_fut}

        <p>Loading...</p>

        {:then polls}

            {#each polls as poll}

                <Poll poll={poll} ></Poll>

            {/each}

        {:catch error}

            <p>{error.message}</p>

    {/await}

</main>

<style>

</style>
