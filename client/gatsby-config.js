module.exports = {
  siteMetadata: {
    title: `TableFlow - Interactive Financial Modeling`,
    description: `Build reusable financial models and instantly see the results`,
    author: `@tableflow`,
  },
  plugins: [
    `gatsby-plugin-react-helmet`,
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `images`,
        path: `${__dirname}/src/images`,
      },
    },
    {
      resolve: `gatsby-plugin-sass`,
      options: {
        includePaths: ["./src/styles"],
      },
    },
    `gatsby-transformer-sharp`,
    `gatsby-plugin-sharp`,
    {
      resolve: `gatsby-plugin-manifest`,
      options: {
        name: `TabeFlow - Interactive Financial Modeling`,
        short_name: `TableFlow`,
        start_url: `/about`,
        background_color: `#FFFFFF`,
        theme_color: `#4386FA`,
        display: `minimal-ui`,
        // This path is relative to the root of the site.
        icon: `src/images/tableflow-icon-512.png`,
      },
    },
    {
      resolve: `gatsby-plugin-create-client-paths`,
      options: { prefixes: [`/studio/*`, `/auth/*`] },
    },
    {
      resolve: `gatsby-plugin-styled-components`,
      options: {},
    },
    {
      resolve: "gatsby-plugin-web-font-loader",
      options: {
        google: {
          families: ["Abel"],
        },
      },
    },
    // To learn more, visit: https://gatsby.dev/offline
    `gatsby-plugin-offline`,
  ],
}
