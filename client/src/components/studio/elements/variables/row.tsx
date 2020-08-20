import React from "react"

import Table from "./table"

const Row = (props: {
  id: string
  value: any
  disabled: boolean
  handleElement: Function
}) => {
  // TODO create a Row type wrapper around a Table
  // it should only allow for a single row to be created
  return (
    <Table
      id={props.id}
      value={props.value}
      disabled={props.disabled}
      handleElement={props.handleElement}
    />
  )
}

export default Row
