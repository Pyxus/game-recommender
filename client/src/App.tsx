import { useState } from "react";
import axios, { AxiosResponse } from "axios";
import "./App.css";

interface Game {
  id: number;
  name: string;
  first_release_date: number;
}

interface RatedGame {
  game?: Game;
  rating: number;
}

interface SearchedGame {
  rated_game: RatedGame;
  search_name: string;
}

function App() {
  const [searchedGames, setSearchedGames] = useState<SearchedGame[]>([]);
  const [focusedSearch, setFocusedSearch] = useState<number>(-1);
  const [searchResults, setSearchResults] = useState<Game[]>([]);
  const [recommendations, setRecommendations] = useState<RatedGame[]>([]);

  const updateSearchResults = async (searchText: string) => {
    if (searchText.length < 2) return;

    try {
      const response: AxiosResponse<Game[]> = await axios.get(
        `http://127.0.0.1:8000/search_game?name=${searchText}`
      );

      setSearchResults(response.data);
    } catch (error) {
      console.error("Error:", error);
    }
  };

  const onSearchNameChanged = async (
    event: React.FormEvent<HTMLInputElement>,
    index: number
  ) => {
    const searchText: string = event.currentTarget.value;

    setSearchedGames(
      searchedGames.map((sg, gIndex) => {
        if (gIndex === index) {
          return { ...sg, search_name: searchText };
        }
        return sg;
      })
    );

    updateSearchResults(searchText);
  };

  const onAddedGameButtonClicked = () => {
    setSearchedGames([
      ...searchedGames,
      { search_name: "", rated_game: { rating: 10 } },
    ]);
  };

  const onDeleteGameButtonClicked = (index: number) => {
    setSearchedGames(searchedGames.filter((item, gIndex) => gIndex !== index));
  };

  const onGameRatingChange = (
    event: React.FormEvent<HTMLInputElement>,
    searchedGame: SearchedGame
  ) => {
    setSearchedGames(
      searchedGames.map((sg) => {
        if (sg === searchedGame) {
          return {
            ...sg,
            rated_game: { rating: parseFloat(event.currentTarget.value) },
          };
        }
        return sg;
      })
    );
  };

  const onGameSelected = (
    searchResultIndex: number,
    searchedGameIndex: number
  ) => {
    const selectedGame = searchResults[searchResultIndex];
    
    setSearchedGames(
      searchedGames.map((sg, index) => {
        if (index === searchedGameIndex) {
          return {
            search_name: selectedGame.name,
            rated_game: { game: selectedGame, rating: sg.rated_game.rating },
          };
        }
        return sg;
      })
    );
    setFocusedSearch(-1);
    setSearchResults([]);
  };

  const onSearchBlur = (index: number) => {
    if (searchedGames[index].rated_game.game === null) {
      searchedGames[index].search_name = "";
      setSearchResults([]);
    }
  };

  const onSubmitClicked = async () => {
    const inputGames = searchedGames.filter((searchedGame) => searchedGame.rated_game.game !== null)
    const rating_by_ids: { [key: number]: number } = {};
    console.log(inputGames)
    inputGames.forEach((searchedGame) => {
      console.log(searchedGame.search_name, ":", searchedGame.rated_game.game?.id)
      if (searchedGame.rated_game.game !== null){
        const game = searchedGame.rated_game.game as Game
        rating_by_ids[game.id] = searchedGame.rated_game.rating;
      }
    })

    if (inputGames.length > 0) {
      try {
        const response: AxiosResponse<RatedGame[]> = await axios.post(
          `http://127.0.0.1:8000/recommend`,
          rating_by_ids,
          { headers: { "Content-Type": "application/json" } }
        );
        setRecommendations(response.data)
      } catch (error) {
        console.error("Error:", error);
      }
    }
  };

  return (
    <>
      <h1>Game Recommender</h1>
      <h2>Select three game titles you've enjoyed to get started!</h2>
      {searchedGames.map((searchedGame: SearchedGame, index: number) => (
        <div key={index}>
          <button onClick={() => onDeleteGameButtonClicked(index)}>X</button>
          <input
            type="number"
            onChange={(event) => onGameRatingChange(event, searchedGame)}
            value={searchedGame.rated_game.rating}
            min={1}
            max={10}
          />
          <input
            type="text"
            placeholder="Enter a game you like"
            value={searchedGame.search_name}
            onChange={(event) => onSearchNameChanged(event, index)}
            onFocus={() => {
              setFocusedSearch(index);
              updateSearchResults(searchedGame.search_name);
            }}
            onBlur={() => onSearchBlur(index)}
          />
          <br />
          {focusedSearch === index && searchResults.length > 0 && (
            <ul>
              {searchResults.map((result, searchIndex) => (
                <li
                  key={searchIndex}
                  onClick={() => onGameSelected(searchIndex, index)}
                >
                  {`${result.name} (${new Date(result.first_release_date * 1000).getFullYear()})`}
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
      <br />
      <button onClick={onAddedGameButtonClicked}>Add Game</button>
      {searchedGames.some(
        (searchedGame) => searchedGame.rated_game.game !== null
      ) && (
        <button
          onClick={() => {
            onSubmitClicked();
          }}
        >
          Submit
        </button>
      )}
      <ol>
      {recommendations.map((ratedGame, index) => (
        <li key={index}>{ratedGame.game?.name}</li>
      ))
      }
      </ol>
    </>
  );
}

export default App;
