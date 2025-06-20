import pandas as pd
import random

def generate_random_frankenstein(n_rows: int, seed: int = 42) -> pd.DataFrame:
    random.seed(seed)

    subjects = [
        "I", "He", "She", "The daemon", "The creature", "Victor", "My soul", "The wretched figure", 
        "My mind", "The lifeless corpse", "They", "The mountains", "The stars"
    ]
    verbs = [
        "beheld", "cursed", "lamented", "trembled at", "reflected on", "sought", "fled from",
        "embraced", "feared", "regarded", "was haunted by", "dared not speak of"
    ]
    emotions = [
        "unspeakable torment", "celestial beauty", "unutterable despair", "eternal dread", 
        "infinite remorse", "profound horror", "ecstasy beyond mortal comprehension",
        "a fevered vision of ruin", "the memory of my crimes", "my abhorrence of life"
    ]
    scenes = [
        "beneath the pale moon", "in the icy desolation of the Alps", "as the storm raged about me",
        "beside the grave of my beloved", "in the stillness of the dawn", 
        "under the blood-red sky", "as night fell upon the lake", "amidst the ruined chapel"
    ]
    modifiers = [
        "with a countenance of misery", "cloaked in dread", "torn between life and death",
        "as if driven by a divine curse", "with every nerve convulsed", 
        "under the weight of sin", "as shadows crept across the land"
    ]
    tails = [
        "and I recoiled from the light of the world.", 
        "which no tongue can utter.", 
        "as if nature herself mourned with me.",
        "leaving me to perish in silence.",
        "with a dreadful certainty of doom.",
        "and I was undone.", 
        "forever marked by my transgression."
    ]

    templates = [
        lambda: f"{random.choice(subjects)} {random.choice(verbs)} {random.choice(emotions)} {random.choice(scenes)}, {random.choice(tails)}",
        lambda: f"{random.choice(subjects)} {random.choice(verbs)} {random.choice(emotions)}, {random.choice(modifiers)}, {random.choice(tails)}",
        lambda: f"In {random.choice(scenes)}, {random.choice(subjects).lower()} {random.choice(verbs)} {random.choice(emotions)}.",
        lambda: f"{random.choice(subjects)} {random.choice(verbs)} not merely {random.choice(emotions)}, but also {random.choice(emotions)}, {random.choice(tails)}",
        lambda: f"{random.choice(emotions).capitalize()} possessed me {random.choice(scenes)}, {random.choice(tails)}"
    ]

    data = []
    for i in range(n_rows):
        line_id = i + 1  # 从1开始的顺序ID
        text = random.choice(templates)()
        data.append((line_id, text))

    df = pd.DataFrame(data, columns=["line_id", "text"])
    return df.sort_values("line_id").reset_index(drop=True)

if __name__ == "__main__":
    df = generate_random_frankenstein(n_rows=4000000)
    df.to_csv("frankenstein_random_fragments.csv", index=False)
