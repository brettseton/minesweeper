using System.Threading.Tasks;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Authentication.Google;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;

namespace backend.Controllers
{
    [Authorize, Route("account")]
    public class AccountController : Controller
    {

        [AllowAnonymous, Route("google-login")]
        public IActionResult GoogleLogin()
        {
            var properties = new AuthenticationProperties { RedirectUri = Request.Query["ReturnURL"] };
            return Challenge(properties, GoogleDefaults.AuthenticationScheme);
        }

        [AllowAnonymous, Route("status")]
        public IActionResult Status()
        {
            return Ok(new { 
                isAuthenticated = User.Identity.IsAuthenticated, 
                name = User.Identity.Name
            });
        }

        [HttpPost, Route("google-logout")]
        public async Task<IActionResult> GoogleLogout()
        {
            await HttpContext.SignOutAsync(CookieAuthenticationDefaults.AuthenticationScheme);
            return Ok(new { Message = "Logged out" });
        }
    }
}
