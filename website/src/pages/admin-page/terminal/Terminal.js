import { React, Component } from 'react'
import TerminalComp from 'terminal-in-react'
import * as AdminApi from '../../../api/AdminApi'
import '../../../index.css'

class Terminal extends Component {
  constructor() {
    super()
    this.sqlExectute = this.sqlExectute.bind(this)
    this.getVariable = this.getVariable.bind(this)
    this.setVariable = this.setVariable.bind(this)
  }
  showMsg = () => 'Hello World'

  sqlExectute(str, print) {
    str.shift()
    const command = str.join(' ')

    AdminApi.executeSql(command).then((res) => print(res))
  }

  getVariable(str, print) {
    str.shift()
    AdminApi.getVariable(str[0]).then((res) => print(res))
  }

  setVariable(str, print) {
    str.shift()
    AdminApi.setVariable(str[0], str[1]).then((res) => print(res))
  }

  render() {
    return (
      <>
        <h1 className="center">Terminal</h1>
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            height: '100vh',
          }}
        >
          <TerminalComp
            color="green"
            backgroundColor="black"
            startState="maximised"
            hideTopBar={true}
            barColor="black"
            style={{ fontWeight: 'bold', fontSize: '1em' }}
            commands={{
              sql: (str, print) => this.sqlExectute(str, print),
              getVariable: (str, print) => this.getVariable(str, print),
              setVariable: (str, print) => this.setVariable(str, print),
            }}
            descriptions={{
              color: false,
              show: false,
              clear: false,
              sql:
                'Exectue a sql command: sql <command>: sql select * from users',
              getVariable:
                'Get server variable, avaliable are: \nis_season\nseason_length\nuser_conf',
              setVariable:
                'Set server variable, avaliable are: \nis_season\nseason_length\nuser_conf',
            }}
            msg="Help for help"
          />
        </div>
      </>
    )
  }
}

export default Terminal
