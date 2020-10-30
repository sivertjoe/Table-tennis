import { React, Component, Suspense } from 'react'
import ReactDOM from 'react-dom'
import { BrowserRouter } from 'react-router-dom'
import './index.css'
import reportWebVitals from './reportWebVitals'
import Routes from './Routes'

class App extends Component {
  render() {
    return (
      <Suspense fallback={<div>Loading...</div>}>
        <Routes />
      </Suspense>
    )
  }
}

ReactDOM.render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
  document.getElementById('root'),
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
