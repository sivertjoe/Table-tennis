// TODO: Can this be done dynamically???
import FirstPlace from './first_place.png'
import SecondPlace from './second_place.png'
import ThirdPlace from './third_place.png'
import Poop from './poop.png'
import Tournament from './first_place.png'
import * as Api from '../api/BaseApi'
import { ApiError } from '../api/ApiErrors'

const images = {
  'first_place.png': FirstPlace,
  'second_place.png': SecondPlace,
  'third_place.png': ThirdPlace,
  'poop.png': Poop,
  'tournament.png': Tournament,
  'assets/tournament_badges/default.png': Api.getImageUrl(
    'tournament_badges/default.png',
  ),
}

export default images
