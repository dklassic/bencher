import React from "react"
import { Box, Button, Icon } from "react-bulma-components"

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faCog } from "@fortawesome/free-solid-svg-icons"

import Table from "./table"
import Function from "./function"
import Row from "./row"

const Variable = (props: {
  element: any
  disabled: { settings: boolean; edit: boolean }
  handleElement: Function
  getElement: Function
}) => {
  function variableSwitch() {
    switch (props.element.type) {
      case "row":
        return (
          <Row
            id={props.element.id}
            value={props.element.value}
            disabled={props.disabled?.edit}
            handleElement={props.handleElement}
          />
        )
      case "table":
        return (
          <Table
            id={props.element.id}
            value={props.element.value}
            disabled={props.disabled?.edit}
            handleElement={props.handleElement}
          />
        )
      case "function":
        return (
          <Function
            id={props.element.id}
            value={props.element.value}
            disabled={props.disabled?.edit}
            handleElement={props.handleElement}
          />
        )
      default:
        return <p>Error: Unknown Element Type</p>
    }
  }

  return (
    <Box>
      {props.element && variableSwitch()}{" "}
      {!props.disabled?.settings && (
        <Button
          color="primary"
          outlined={true}
          size="small"
          fullwidth={true}
          title="Settings"
          onClick={(event: any) => {
            event.preventDefault()
            console.log("TODO edit settings")
          }}
        >
          <Icon className="primary">
            <FontAwesomeIcon icon={faCog} size="1x" />
          </Icon>
          <span>Settings</span>
        </Button>
      )}
    </Box>
  )
}

export default Variable
