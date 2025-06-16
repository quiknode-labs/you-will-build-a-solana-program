import cliConfetti from "cli-confetti";
import CliUpdate from "cli-update";

cliConfetti({}, function (error, confetti) {
  if (error) throw error;
  CliUpdate.render(confetti);
});
